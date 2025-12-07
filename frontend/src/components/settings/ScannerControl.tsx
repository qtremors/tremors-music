```typescript
// Scanner control with progress tracking
import { useState, useEffect, useRef } from 'react';
import { RefreshCw, StopCircle, CheckCircle, AlertCircle, Clock } from 'lucide-react';
import api from '../../lib/api';
import { Card } from '../common/Card';
import { Button } from '../common/Button';

interface LastScanResult {
  files_processed: number;
  songs_added: number;
  errors: number;
  duration: number;
  completed_at: string;
}

interface ScanProgress {
  is_scanning: boolean;
  files_processed: number;
  songs_added: number;
  errors: number;
  current_file: string;
  start_time: number | null;
  last_scan_result: LastScanResult | null;
}

export function ScannerControl() {
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<ScanProgress>({
    is_scanning: false,
    files_processed: 0,
    songs_added: 0,
    errors: 0,
    current_file: '',
    start_time: null,
    last_scan_result: null
  });
  const pollInterval = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    // Check initial scan status
    checkScanStatus();
    
    return () => {
      if (pollInterval.current) {
        clearInterval(pollInterval.current);
      }
    };
  }, []);

  const checkScanStatus = async () => {
    try {
      const res = await api.get<ScanProgress>('/library/scan/status');
      setProgress(res.data);
      setIsScanning(res.data.is_scanning);
      setError(null);
      
      if (res.data.is_scanning && !pollInterval.current) {
        startPolling();
      } else if (!res.data.is_scanning && pollInterval.current) {
        stopPolling();
      }
    } catch (e: any) {
      console.error('Failed to get scan status:', e);
      setError(e.response?.data?.detail || 'Failed to connect to scanner');
    }
  };

  const startPolling = () => {
    pollInterval.current = setInterval(checkScanStatus, 500);
  };

  const stopPolling = () => {
    if (pollInterval.current) {
      clearInterval(pollInterval.current);
      pollInterval.current = null;
    }
  };

  const handleStartScan = async () => {
    try {
      setError(null);
      setIsScanning(true);
      await api.post('/library/scan');
      startPolling();
    } catch (e: any) {
      console.error('Failed to start scan:', e);
      setError(e.response?.data?.detail || 'Failed to start scan');
      setIsScanning(false);
    }
  };

  const handleStopScan = async () => {
    try {
      // TODO: Implement stop endpoint
      // await api.post('/library/scan/stop');
      stopPolling();
      setIsScanning(false);
    } catch (e) {
      console.error('Failed to stop scan:', e);
    }
  };

  const getElapsedTime = () => {
    if (!progress.start_time) return '0s';
    const elapsed = Math.floor((Date.now() - progress.start_time) / 1000);
    const mins = Math.floor(elapsed / 60);
    const secs = elapsed % 60;
    return mins > 0 ? `${ mins }m ${ secs } s` : `${ secs } s`;
  };

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return mins > 0 ? `${ mins }m ${ secs } s` : `${ secs } s`;
  };

  const formatDate = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleString();
  };

  return (
    <Card className="mt-4">
      <div className="space-y-4">
        <h4 className="text-sm font-semibold text-apple-subtext uppercase tracking-wider">Library Scanner</h4>
        
        {/* Error Display */}
        {error && (
          <div className="flex items-start gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <AlertCircle size={18} className="text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <div className="text-sm font-medium text-red-700 dark:text-red-300">Scan Error</div>
              <div className="text-xs text-red-600 dark:text-red-400 mt-1">{error}</div>
            </div>
          </div>
        )}
        
        {/* Scan Button */}
        {!isScanning ? (
          <Button
            onClick={handleStartScan}
            variant="primary"
            className="w-full"
            disabled={isScanning}
          >
            <RefreshCw size={18} />
            Scan All Folders
          </Button>
        ) : (
          <Button
            onClick={handleStopScan}
            variant="danger"
            className="w-full"
          >
            <StopCircle size={18} />
            Stop Scanning
          </Button>
        )}

        {/* Progress Display */}
        {isScanning && (
          <div className="space-y-3 p-4 bg-gray-50 dark:bg-black/20 rounded-lg border border-gray-200 dark:border-white/10">
            {/* Progress Bar */}
            <div className="space-y-1">
              <div className="flex justify-between text-xs text-apple-subtext">
                <span>Scanning...</span>
                <span>{getElapsedTime()}</span>
              </div>
              <div className="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-apple-accent transition-all duration-300 rounded-full"
                  style={{ width: '100%' }}
                >
                  <div className="h-full w-full bg-gradient-to-r from-transparent via-white/30 to-transparent animate-pulse" />
                </div>
              </div>
            </div>

            {/* Current File */}
            {progress.current_file && (
              <div className="text-xs text-apple-subtext truncate">
                📁 {progress.current_file}
              </div>
            )}

            {/* Statistics */}
            <div className="grid grid-cols-3 gap-3 pt-2 border-t border-gray-200 dark:border-white/10">
              <div className="text-center">
                <div className="text-lg font-semibold text-apple-text">{progress.files_processed}</div>
                <div className="text-xs text-apple-subtext">Files</div>
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold text-green-500">{progress.songs_added}</div>
                <div className="text-xs text-apple-subtext">Added</div>
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold text-red-500">{progress.errors}</div>
                <div className="text-xs text-apple-subtext">Errors</div>
              </div>
            </div>
          </div>
        )}

        {/* Last Scan Result - Persistent */}
        {!isScanning && progress.last_scan_result && (
          <div className="space-y-2 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <CheckCircle size={18} className="text-blue-600 dark:text-blue-400" />
                <span className="text-sm font-semibold text-blue-700 dark:text-blue-300">Last Scan Complete</span>
              </div>
              <div className="flex items-center gap-1 text-xs text-blue-600 dark:text-blue-400">
                <Clock size={12} />
                {formatDuration(progress.last_scan_result.duration)}
              </div>
            </div>
            
            <div className="grid grid-cols-3 gap-2 mt-2">
              <div className="text-center p-2 bg-white/50 dark:bg-black/20 rounded">
                <div className="text-sm font-bold text-blue-700 dark:text-blue-300">{progress.last_scan_result.files_processed}</div>
                <div className="text-xs text-blue-600 dark:text-blue-400">Files</div>
              </div>
              <div className="text-center p-2 bg-white/50 dark:bg-black/20 rounded">
                <div className="text-sm font-bold text-green-600 dark:text-green-400">{progress.last_scan_result.songs_added}</div>
                <div className="text-xs text-blue-600 dark:text-blue-400">Added</div>
              </div>
              <div className="text-center p-2 bg-white/50 dark:bg-black/20 rounded">
                <div className="text-sm font-bold text-red-600 dark:text-red-400">{progress.last_scan_result.errors}</div>
                <div className="text-xs text-blue-600 dark:text-blue-400">Errors</div>
              </div>
            </div>
            
            <div className="text-xs text-blue-600 dark:text-blue-400 mt-2">
              Completed: {formatDate(progress.last_scan_result.completed_at)}
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}
```
