import { createRoot } from 'react-dom/client';
import App from './App';
import './styles.css';

// No React.StrictMode: its dev-only effect double-run would move dialog focus.
createRoot(document.getElementById('root') as HTMLElement).render(<App />);

