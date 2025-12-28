import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { oauthService } from '../../services/oauth';

export default function OAuthCallback() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(true);

  useEffect(() => {
    const handleCallback = async () => {
      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const errorParam = searchParams.get('error');
      const errorDescription = searchParams.get('error_description');

      // Handle OAuth error response
      if (errorParam) {
        setError(errorDescription || `Authentication error: ${errorParam}`);
        setIsProcessing(false);
        setTimeout(() => navigate('/login'), 3000);
        return;
      }

      // Validate required parameters
      if (!code || !state) {
        setError('Missing authorization code or state');
        setIsProcessing(false);
        setTimeout(() => navigate('/login'), 3000);
        return;
      }

      try {
        // Exchange code for token
        await oauthService.handleCallback(code, state);

        // Success - redirect to dashboard
        navigate('/dashboard', { replace: true });
      } catch (err) {
        console.error('OAuth callback error:', err);
        setError(err instanceof Error ? err.message : 'Authentication failed');
        setIsProcessing(false);
        setTimeout(() => navigate('/login'), 3000);
      }
    };

    handleCallback();
  }, [searchParams, navigate]);

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
    }}>
      {isProcessing ? (
        <>
          <div style={{
            width: '50px',
            height: '50px',
            border: '3px solid #333',
            borderTop: '3px solid #4CAF50',
            borderRadius: '50%',
            animation: 'spin 1s linear infinite',
          }} />
          <style>{`
            @keyframes spin {
              0% { transform: rotate(0deg); }
              100% { transform: rotate(360deg); }
            }
          `}</style>
          <p style={{ marginTop: '20px', fontSize: '18px' }}>
            Completing authentication...
          </p>
        </>
      ) : error ? (
        <>
          <div style={{
            fontSize: '48px',
            marginBottom: '20px',
            color: '#f44336',
          }}>
            ✕
          </div>
          <h1 style={{ fontSize: '24px', marginBottom: '10px' }}>
            Authentication Failed
          </h1>
          <p style={{ fontSize: '16px', color: '#888', maxWidth: '400px', textAlign: 'center' }}>
            {error}
          </p>
          <p style={{ fontSize: '14px', color: '#666', marginTop: '20px' }}>
            Redirecting to login...
          </p>
        </>
      ) : null}
    </div>
  );
}
