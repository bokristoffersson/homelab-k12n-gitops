# iOS/macOS Native App Development Guide

This guide covers building native iOS/macOS apps for the Homelab API.

## API Specifications

| Service | Spec File | Protocol |
|---------|-----------|----------|
| Homelab API | `homelab-api.yaml` | OpenAPI 3.0 (REST) |
| Heatpump Settings API | `heatpump-settings-api.yaml` | OpenAPI 3.0 (REST) |
| Energy WebSocket | `energy-ws.yaml` | AsyncAPI 2.6 (WebSocket) |

## Swift Code Generation

### Option 1: Swift OpenAPI Generator (Recommended)

Apple's official OpenAPI generator for Swift. Generates type-safe, async/await Swift code.

```bash
# Install the generator
brew install swift-openapi-generator

# Generate client code
swift-openapi-generator generate \
  --mode types+client \
  --output-directory Sources/HomelabAPI \
  homelab-api.yaml

swift-openapi-generator generate \
  --mode types+client \
  --output-directory Sources/HeatpumpSettingsAPI \
  heatpump-settings-api.yaml
```

Add to your `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/apple/swift-openapi-runtime", from: "1.0.0"),
    .package(url: "https://github.com/apple/swift-openapi-urlsession", from: "1.0.0"),
]
```

### Option 2: OpenAPI Generator (Alternative)

More mature, supports many languages. Use the `swift5` generator.

```bash
# Install
brew install openapi-generator

# Generate Swift 5 client
openapi-generator generate \
  -i homelab-api.yaml \
  -g swift5 \
  -o HomelabAPIClient \
  --additional-properties=projectName=HomelabAPI,responseAs=AsyncAwait

openapi-generator generate \
  -i heatpump-settings-api.yaml \
  -g swift5 \
  -o HeatpumpSettingsAPIClient \
  --additional-properties=projectName=HeatpumpSettingsAPI,responseAs=AsyncAwait
```

### Option 3: CreateAPI

Flexible Swift code generator with excellent customization.

```bash
# Install
brew install create-api

# Generate
create-api generate homelab-api.yaml --output Sources/HomelabAPI
create-api generate heatpump-settings-api.yaml --output Sources/HeatpumpSettingsAPI
```

## Authentication

All APIs use JWT tokens from Authentik (OIDC provider).

### PKCE Authorization Flow

For native apps, use **Authorization Code with PKCE** (Proof Key for Code Exchange):

```swift
import AuthenticationServices

class AuthManager: NSObject, ASWebAuthenticationPresentationContextProviding {
    private let clientId = "your-client-id"
    private let redirectUri = "homelab://oauth/callback"
    private let authorizationEndpoint = "https://auth.k12n.com/application/o/authorize/"
    private let tokenEndpoint = "https://auth.k12n.com/application/o/token/"

    func authenticate() async throws -> OAuthTokens {
        // Generate PKCE values
        let codeVerifier = generateCodeVerifier()
        let codeChallenge = generateCodeChallenge(from: codeVerifier)

        // Build authorization URL
        var components = URLComponents(string: authorizationEndpoint)!
        components.queryItems = [
            URLQueryItem(name: "client_id", value: clientId),
            URLQueryItem(name: "redirect_uri", value: redirectUri),
            URLQueryItem(name: "response_type", value: "code"),
            URLQueryItem(name: "scope", value: "openid profile email"),
            URLQueryItem(name: "code_challenge", value: codeChallenge),
            URLQueryItem(name: "code_challenge_method", value: "S256"),
        ]

        // Present authentication session
        let callbackURL = try await withCheckedThrowingContinuation { continuation in
            let session = ASWebAuthenticationSession(
                url: components.url!,
                callbackURLScheme: "homelab"
            ) { url, error in
                if let error = error {
                    continuation.resume(throwing: error)
                } else if let url = url {
                    continuation.resume(returning: url)
                }
            }
            session.presentationContextProvider = self
            session.prefersEphemeralWebBrowserSession = false
            session.start()
        }

        // Extract authorization code
        let code = URLComponents(url: callbackURL, resolvingAgainstBaseURL: false)?
            .queryItems?
            .first(where: { $0.name == "code" })?
            .value

        guard let authCode = code else {
            throw AuthError.noAuthorizationCode
        }

        // Exchange code for tokens
        return try await exchangeCodeForTokens(authCode, codeVerifier: codeVerifier)
    }

    private func generateCodeVerifier() -> String {
        var buffer = [UInt8](repeating: 0, count: 32)
        _ = SecRandomCopyBytes(kSecRandomDefault, buffer.count, &buffer)
        return Data(buffer).base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }

    private func generateCodeChallenge(from verifier: String) -> String {
        let data = Data(verifier.utf8)
        var hash = [UInt8](repeating: 0, count: Int(CC_SHA256_DIGEST_LENGTH))
        data.withUnsafeBytes {
            _ = CC_SHA256($0.baseAddress, CC_LONG(data.count), &hash)
        }
        return Data(hash).base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }

    func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        return ASPresentationAnchor()
    }
}
```

### Token Storage

Store tokens securely in Keychain:

```swift
import Security

class TokenStorage {
    private let service = "com.homelab.api"

    func saveTokens(_ tokens: OAuthTokens) throws {
        let data = try JSONEncoder().encode(tokens)

        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: "oauth_tokens",
            kSecValueData as String: data,
        ]

        SecItemDelete(query as CFDictionary)
        let status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else {
            throw KeychainError.saveFailed(status)
        }
    }

    func loadTokens() throws -> OAuthTokens? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: "oauth_tokens",
            kSecReturnData as String: true,
        ]

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess, let data = result as? Data else {
            return nil
        }

        return try JSONDecoder().decode(OAuthTokens.self, from: data)
    }
}
```

## API Client Setup

### REST Client Example

```swift
import Foundation

actor HomelabAPIClient {
    private let baseURL = URL(string: "https://api.k12n.com")!
    private let session: URLSession
    private let tokenProvider: TokenProvider

    init(tokenProvider: TokenProvider) {
        self.tokenProvider = tokenProvider

        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 30
        self.session = URLSession(configuration: config)
    }

    func getEnergyLatest() async throws -> EnergyLatest {
        let request = try await buildRequest(path: "/api/v1/energy/latest")
        let (data, response) = try await session.data(for: request)
        try validateResponse(response)
        return try JSONDecoder.iso8601.decode(EnergyLatest.self, from: data)
    }

    func getHeatpumpLatest(deviceId: String? = nil) async throws -> HeatpumpLatest {
        var path = "/api/v1/heatpump/latest"
        if let deviceId = deviceId {
            path += "?device_id=\(deviceId)"
        }
        let request = try await buildRequest(path: path)
        let (data, response) = try await session.data(for: request)
        try validateResponse(response)
        return try JSONDecoder.iso8601.decode(HeatpumpLatest.self, from: data)
    }

    func getTemperatureAllLatest() async throws -> [TemperatureLatest] {
        let request = try await buildRequest(path: "/api/v1/temperature/all-latest")
        let (data, response) = try await session.data(for: request)
        try validateResponse(response)
        return try JSONDecoder.iso8601.decode([TemperatureLatest].self, from: data)
    }

    private func buildRequest(path: String, method: String = "GET") async throws -> URLRequest {
        var request = URLRequest(url: baseURL.appendingPathComponent(path))
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Accept")

        let token = try await tokenProvider.getValidToken()
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")

        return request
    }

    private func validateResponse(_ response: URLResponse) throws {
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }

        switch httpResponse.statusCode {
        case 200..<300:
            return
        case 401:
            throw APIError.unauthorized
        case 404:
            throw APIError.notFound
        default:
            throw APIError.serverError(httpResponse.statusCode)
        }
    }
}

extension JSONDecoder {
    static let iso8601: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }()
}
```

## WebSocket Client

For real-time energy data streaming:

```swift
import Foundation

actor EnergyWebSocketClient {
    private let url: URL
    private var webSocketTask: URLWebSocketTask?
    private let tokenProvider: TokenProvider

    var onEnergyData: ((EnergyData) -> Void)?
    var onError: ((Error) -> Void)?

    init(tokenProvider: TokenProvider) {
        self.tokenProvider = tokenProvider
        self.url = URL(string: "wss://energy-ws.k12n.com/ws/energy")!
    }

    func connect() async throws {
        let token = try await tokenProvider.getValidToken()
        var components = URLComponents(url: url, resolvingAgainstBaseURL: false)!
        components.queryItems = [URLQueryItem(name: "token", value: token)]

        webSocketTask = URLSession.shared.webSocketTask(with: components.url!)
        webSocketTask?.resume()

        // Subscribe to energy stream
        try await send(SubscribeMessage(streams: ["energy"]))

        // Start receiving messages
        Task {
            await receiveMessages()
        }
    }

    func disconnect() {
        webSocketTask?.cancel(with: .goingAway, reason: nil)
        webSocketTask = nil
    }

    private func send<T: Encodable>(_ message: T) async throws {
        let data = try JSONEncoder().encode(message)
        try await webSocketTask?.send(.data(data))
    }

    private func receiveMessages() async {
        while let task = webSocketTask {
            do {
                let message = try await task.receive()
                switch message {
                case .data(let data):
                    try handleMessage(data)
                case .string(let text):
                    if let data = text.data(using: .utf8) {
                        try handleMessage(data)
                    }
                @unknown default:
                    break
                }
            } catch {
                onError?(error)
                break
            }
        }
    }

    private func handleMessage(_ data: Data) throws {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        // Decode base message to check type
        let base = try decoder.decode(BaseMessage.self, from: data)

        switch base.type {
        case "data":
            let dataMessage = try decoder.decode(DataMessage.self, from: data)
            if dataMessage.stream == "energy" {
                onEnergyData?(dataMessage.data)
            }
        case "pong":
            // Handle pong
            break
        case "error":
            let errorMessage = try decoder.decode(ErrorMessage.self, from: data)
            onError?(WebSocketError.serverError(errorMessage.message))
        default:
            break
        }
    }

    func sendPing() async throws {
        try await send(PingMessage())
    }
}

// Message types
struct SubscribeMessage: Codable {
    let type = "subscribe"
    let streams: [String]
}

struct PingMessage: Codable {
    let type = "ping"
}

struct BaseMessage: Codable {
    let type: String
}

struct DataMessage: Codable {
    let type: String
    let stream: String
    let timestamp: Date
    let data: EnergyData
}

struct ErrorMessage: Codable {
    let type: String
    let message: String
    let code: String
}
```

## SwiftUI Integration

### Example ViewModel

```swift
import SwiftUI
import Combine

@MainActor
class DashboardViewModel: ObservableObject {
    @Published var energyLatest: EnergyLatest?
    @Published var heatpumpLatest: HeatpumpLatest?
    @Published var temperatures: [TemperatureLatest] = []
    @Published var isLoading = false
    @Published var error: Error?

    private let api: HomelabAPIClient
    private let wsClient: EnergyWebSocketClient

    init(tokenProvider: TokenProvider) {
        self.api = HomelabAPIClient(tokenProvider: tokenProvider)
        self.wsClient = EnergyWebSocketClient(tokenProvider: tokenProvider)
    }

    func loadData() async {
        isLoading = true
        error = nil

        do {
            async let energy = api.getEnergyLatest()
            async let heatpump = api.getHeatpumpLatest()
            async let temps = api.getTemperatureAllLatest()

            let (e, h, t) = try await (energy, heatpump, temps)

            energyLatest = e
            heatpumpLatest = h
            temperatures = t
        } catch {
            self.error = error
        }

        isLoading = false
    }

    func startRealTimeUpdates() async {
        do {
            try await wsClient.connect()
            await wsClient.onEnergyData = { [weak self] data in
                Task { @MainActor in
                    self?.energyLatest = EnergyLatest(
                        ts: Date(),
                        consumptionTotalW: data.consumptionTotalW,
                        consumptionL1ActualW: data.consumptionL1ActualW,
                        consumptionL2ActualW: data.consumptionL2ActualW,
                        consumptionL3ActualW: data.consumptionL3ActualW
                    )
                }
            }
        } catch {
            self.error = error
        }
    }

    func stopRealTimeUpdates() async {
        await wsClient.disconnect()
    }
}
```

### Example View

```swift
import SwiftUI

struct DashboardView: View {
    @StateObject private var viewModel: DashboardViewModel

    init(tokenProvider: TokenProvider) {
        _viewModel = StateObject(wrappedValue: DashboardViewModel(tokenProvider: tokenProvider))
    }

    var body: some View {
        NavigationStack {
            List {
                if let energy = viewModel.energyLatest {
                    Section("Energy") {
                        LabeledContent("Total Power", value: "\(energy.consumptionTotalW ?? 0) W")
                        LabeledContent("L1", value: "\(energy.consumptionL1ActualW ?? 0) W")
                        LabeledContent("L2", value: "\(energy.consumptionL2ActualW ?? 0) W")
                        LabeledContent("L3", value: "\(energy.consumptionL3ActualW ?? 0) W")
                    }
                }

                if let heatpump = viewModel.heatpumpLatest {
                    Section("Heatpump") {
                        LabeledContent("Compressor", value: heatpump.compressorOn == true ? "On" : "Off")
                        LabeledContent("Outdoor", value: formatTemp(heatpump.outdoorTemp))
                        LabeledContent("Supply Line", value: formatTemp(heatpump.supplylineTemp))
                        LabeledContent("Hot Water", value: formatTemp(heatpump.hotwaterTemp))
                    }
                }

                Section("Temperature Sensors") {
                    ForEach(viewModel.temperatures, id: \.location) { temp in
                        LabeledContent(temp.location ?? "Unknown") {
                            VStack(alignment: .trailing) {
                                Text(String(format: "%.1f°C", temp.temperatureC ?? 0))
                                Text(String(format: "%.0f%%", temp.humidity ?? 0))
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
            .navigationTitle("Dashboard")
            .refreshable {
                await viewModel.loadData()
            }
            .task {
                await viewModel.loadData()
                await viewModel.startRealTimeUpdates()
            }
        }
    }

    private func formatTemp(_ value: Int16?) -> String {
        guard let value = value else { return "N/A" }
        return String(format: "%.1f°C", Double(value) / 10.0)
    }
}
```

## Xcode Project Setup

1. Create a new Xcode project (App or Multiplatform)
2. Add Swift Package dependencies:
   - `swift-openapi-runtime` (if using Swift OpenAPI Generator)
   - `swift-openapi-urlsession` (if using Swift OpenAPI Generator)
3. Generate API clients from OpenAPI specs
4. Configure URL schemes for OAuth callback (`homelab://`)
5. Add Keychain capability for token storage

### Info.plist Configuration

```xml
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>homelab</string>
        </array>
        <key>CFBundleURLName</key>
        <string>OAuth Callback</string>
    </dict>
</array>
```

## Authentik Client Configuration

Register a new OAuth2 application in Authentik:

1. Go to Authentik Admin > Applications > Providers
2. Create new OAuth2/OpenID Provider:
   - Name: `Homelab iOS/macOS App`
   - Client Type: `Public` (for native apps)
   - Redirect URIs: `homelab://oauth/callback`
   - Scopes: `openid profile email`
3. Create Application linked to the provider
4. Note the Client ID for your app configuration

## API Endpoints Summary

### Homelab API (Read-only)

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/energy/latest` | Latest energy reading |
| `GET /api/v1/energy/hourly-total` | Current hour total |
| `GET /api/v1/energy/history` | Historical data |
| `GET /api/v1/energy/daily-summary` | Daily aggregates |
| `GET /api/v1/heatpump/latest` | Latest heatpump data |
| `GET /api/v1/heatpump/daily-summary` | Daily statistics |
| `GET /api/v1/temperature/latest` | Single sensor reading |
| `GET /api/v1/temperature/all-latest` | All sensors |
| `GET /api/v1/temperature/history` | Sensor history |

### Heatpump Settings API

| Endpoint | Description |
|----------|-------------|
| `GET /api/v1/heatpump/settings` | All device settings |
| `GET /api/v1/heatpump/settings/{device_id}` | Device settings |
| `PATCH /api/v1/heatpump/settings/{device_id}` | Update settings |
| `GET /api/v1/heatpump/settings/{device_id}/outbox` | Command status |

### Energy WebSocket

| Message Type | Direction | Description |
|--------------|-----------|-------------|
| `subscribe` | Client → Server | Subscribe to streams |
| `unsubscribe` | Client → Server | Unsubscribe |
| `ping` | Client → Server | Keepalive |
| `data` | Server → Client | Real-time data |
| `pong` | Server → Client | Ping response |
| `error` | Server → Client | Error notification |
