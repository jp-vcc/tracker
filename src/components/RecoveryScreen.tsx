import { useTracker } from '../state';
import type { RecoveryResult } from '../types';

/** Shown instead of the app when the data file is corrupt, too new, or unreadable. */
export function RecoveryScreen({ result }: { result: RecoveryResult }) {
  const { actions } = useTracker();

  if (result.status === 'io_error') {
    return (
      <main className="recovery">
        <h1>Your data could not be opened</h1>
        <p>{result.message}</p>
        <p>Your data file was not changed. Close any program that may be using it, then try again.</p>
        <button type="button" className="primary" onClick={() => void actions.retryLoad()}>
          Retry
        </button>
      </main>
    );
  }

  return (
    <main className="recovery">
      {result.status === 'corrupt' ? (
        <>
          <h1>Your data file could not be read</h1>
          <p>{result.reason}</p>
          <p>{`A backup copy of the unreadable file was saved at: ${result.backupPath}`}</p>
        </>
      ) : (
        <>
          <h1>Your data was saved by a newer version</h1>
          <p>{`The data file uses format version ${result.version}, which this version cannot read. Please update the app.`}</p>
          <p>If you start with empty data instead, a backup copy of the existing file is saved first.</p>
        </>
      )}
      <p>The original file stays untouched until you choose to start over.</p>
      <button type="button" className="primary" onClick={() => void actions.startFresh()}>
        Start with empty data
      </button>
    </main>
  );
}

