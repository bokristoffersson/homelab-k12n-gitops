import { NavLink, Outlet } from 'react-router-dom';
import { useTheme } from '../../hooks/useTheme';
import './Layout.css';

export default function Layout() {
  const { theme, toggleTheme } = useTheme();

  return (
    <div className="layout">
      <nav className="navbar">
        <div className="navbar-content">
          <h1 className="navbar-title">Heatpump Monitor</h1>
          <div className="navbar-tabs">
            <NavLink
              to="/dashboard"
              className={({ isActive }) => isActive ? 'tab active' : 'tab'}
            >
              Dashboard
            </NavLink>
            <NavLink
              to="/settings"
              className={({ isActive }) => isActive ? 'tab active' : 'tab'}
            >
              Settings
            </NavLink>
            <NavLink
              to="/plugs"
              className={({ isActive }) => isActive ? 'tab active' : 'tab'}
            >
              Plugs
            </NavLink>
          </div>
          <div className="navbar-actions">
            <span className="last-update">
              {new Date().toLocaleTimeString()}
            </span>
            <button
              className="theme-toggle"
              onClick={toggleTheme}
              aria-label="Toggle theme"
            >
              {theme === 'light' ? '🌙' : '☀️'}
            </button>
          </div>
        </div>
      </nav>
      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
