import { oauthService } from '../../services/oauth';

export default function LoginOAuth() {
  const handleLogin = async () => {
    await oauthService.login();
  };

  return (
    <div style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '100vh',
      backgroundColor: '#0a0a0a',
      color: '#fff',
      fontFamily: 'system-ui, -apple-system, sans-serif',
      padding: '20px',
    }}>
      <div style={{
        maxWidth: '400px',
        width: '100%',
        backgroundColor: '#1a1a1a',
        borderRadius: '12px',
        padding: '40px',
        boxShadow: '0 4px 6px rgba(0, 0, 0, 0.3)',
      }}>
        <h1 style={{
          fontSize: '28px',
          marginBottom: '10px',
          textAlign: 'center',
        }}>
          Heatpump Dashboard
        </h1>
        <p style={{
          fontSize: '14px',
          color: '#888',
          textAlign: 'center',
          marginBottom: '30px',
        }}>
          Sign in to access your dashboard
        </p>

        <button
          onClick={handleLogin}
          style={{
            width: '100%',
            padding: '14px',
            fontSize: '16px',
            fontWeight: '600',
            color: '#fff',
            backgroundColor: '#4CAF50',
            border: 'none',
            borderRadius: '8px',
            cursor: 'pointer',
            transition: 'background-color 0.2s',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = '#45a049';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = '#4CAF50';
          }}
        >
          Sign in with Authentik
        </button>

        <div style={{
          marginTop: '30px',
          padding: '15px',
          backgroundColor: '#2a2a2a',
          borderRadius: '8px',
          fontSize: '13px',
          color: '#aaa',
        }}>
          <p style={{ margin: '0 0 8px 0', fontWeight: '600', color: '#fff' }}>
            Secure OAuth2 Authentication
          </p>
          <ul style={{ margin: 0, paddingLeft: '20px' }}>
            <li>Authenticated via Authentik</li>
            <li>Uses OAuth2 with PKCE</li>
            <li>Your credentials never leave Authentik</li>
          </ul>
        </div>
      </div>

      <p style={{
        marginTop: '30px',
        fontSize: '12px',
        color: '#666',
      }}>
        Powered by Kong API Gateway + Authentik
      </p>
    </div>
  );
}
