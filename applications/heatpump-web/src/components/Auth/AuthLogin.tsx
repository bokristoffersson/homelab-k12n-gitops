import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

/**
 * AuthLogin - handles post-OIDC redirect
 *
 * This component is reached after traefikoidc completes the OIDC flow.
 * The session cookie has been set by traefikoidc, so we just need to
 * redirect the user to their intended destination.
 */
export default function AuthLogin() {
  const navigate = useNavigate();

  useEffect(() => {
    // Get stored redirect location or default to dashboard
    const redirectPath = sessionStorage.getItem('auth_redirect') || '/dashboard';
    sessionStorage.removeItem('auth_redirect');

    // Navigate to the intended destination
    navigate(redirectPath, { replace: true });
  }, [navigate]);

  // Brief loading state while redirecting
  return (
    <div style={{
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      height: '100vh',
      fontSize: '1.2rem',
      color: '#666'
    }}>
      Completing login...
    </div>
  );
}
