# Migration: Authentik → Authelia

Brief till Claude Code. Det här är en **delegeringsbeskrivning**, inte en tutorial.
Du kan implementationen bättre än den här texten — håll dig till aktuell Authelia-dokumentation
för konkreta fält, chart-versioner och kommandon. Det här dokumentet definierar *mål, beslut,
constraints och acceptans*. Resten är ditt.

## Mål

Ersätt Authentik med Authelia som OIDC-provider i klustret, **med bibehållen funktion**.
Alla befintliga relying parties ska logga in som tidigare när migreringen är klar, och
Authentik ska vara borttaget ur repot och klustret.

Färdig = varje app som idag autentiserar mot Authentik autentiserar mot Authelia,
inget tillstånd hanteras utanför Git, och Authentik-resurserna är borta.

## Hårda krav

- **FluxCD** är enda deploy-mekanismen. Inga `kubectl apply` för manuell drift, ingen ArgoCD.
  Allt går via Git → Flux-reconcile (HelmRelease eller Kustomization, du väljer det som passar repots befintliga mönster).
- **Config-as-code, fullt ut.** Authelia har ingen admin-GUI och det är poängen. Hela
  konfigurationen (clients, access control, scopes, claims, token-livslängder) ska ligga i Git.
  Inget klustertillstånd får skapas via UI eller imperativa kommandon.
- **Secrets committas aldrig i klartext.** Återanvänd repots befintliga secret-mönster
  (sannolikt SOPS eller sealed-secrets — verifiera vilket). Hashade OIDC client secrets får
  ligga i committad config; krypto-nycklar och session-/storage-secrets injiceras från Kubernetes Secrets.
- **Opaque access tokens är default och ska behållas.** Byt bara en enskild client till JWT
  om just den appen kräver det (se "Bevara" nedan).
- **Ingen nedtid på inloggning.** Authentik körs kvar tills Authelia är verifierad.

## Tillvägagångssätt (faser)

1. **Inventera.** Läs nuvarande Authentik-resurser i repot och i klustret. Producera en
   fullständig lista över relying parties: `client_id`, redirect URIs, begärda scopes,
   och vilka claims/grupper varje app använder för authorization.
2. **Mappa.** Översätt varje provider/application till en Authelia-client. Behåll `client_id`
   där appens egen config är svår att ändra. Skapa en mappningstabell (Authentik → Authelia) i en
   markdown-fil i repot som en del av PR:en.
3. **Deploya parallellt.** Stå upp Authelia via Flux vid sidan av Authentik. Lös krypto-nycklar
   och secrets enligt repots mönster. Reconcile och verifiera att Authelia blir healthy.
4. **Verifiera per client.** Testa inloggning app för app mot Authelia innan cut-over.
5. **Cut over.** Peka om apparna till Authelias issuer/endpoints.
6. **Riv Authentik.** Ta bort Authentik-resurser ur repot (server, worker, ev. egen Postgres/Redis
   om inget annat använder dem) och låt Flux städa.

Arbeta i en branch och leverera som PR. Låt Flux reconcila branchen där det går; bekräfta
reconcile-status snarare än att anta.

## Bevara explicit (annars går funktion sönder)

- **Redirect URIs** måste matcha exakt — minsta avvikelse bryter login.
- **Scopes och claims som apparna gör authz på.** Om en app kollar t.ex. en grupp-claim måste
  samma claim med samma värden produceras av Authelia. Detta är den vanligaste tysta regressionen.
- **Refresh tokens.** Clients som idag får refresh token förutsätter `offline_access` — säkerställ
  att scopet finns kvar för dem.
- **Användare och grupper.** Bestäm backend (file-backend i YAML, GitOps-bart, eller LDAP/LLDAP)
  och migrera användarnamn och gruppmedlemskap så att authz-besluten blir identiska.
- **JWT-beroende access tokens.** Om någon app idag dekodar access token som JWT: sätt JWT-läge
  enbart för den clienten. Övriga ska vara opaque.

## Definition of Done

- Varje inventerad relying party loggar in mot Authelia, inkl. korrekta grupp-/claim-baserade authz-beslut.
- Refresh token-flödet fungerar för de clients som använde det.
- Inget Authentik kvar i repo eller kluster.
- Hela auth-konfigurationen är deklarativ i Git; inga secrets i klartext.
- Mappningstabellen (Authentik → Authelia) finns committad.
- Flux reconcilar rent utan drift.

## Vad du *inte* behöver från mig

Välj själv chart vs raka manifests, Authelia-version, storage-backend (SQLite vid en replika,
annars Postgres för delat session-state över noderna), och exakta config-fält — slå upp aktuell
Authelia-dokumentation. Föredra det enklaste som uppfyller kraven framför det mest konfigurerbara.
