import { useEffect, useState } from 'react';

type ToastMessage = { id: number; text: string; type?: 'success' | 'error' | 'info' };

let toastId = 0;
const listeners = new Set<() => void>();
let currentToast: ToastMessage | null = null;

export function showToast(text: string, type: 'success' | 'error' | 'info' = 'success', durationMs = 2500) {
  currentToast = { id: ++toastId, text, type };
  listeners.forEach(fn => fn());
  if (durationMs > 0) {
    setTimeout(() => {
      if (currentToast?.id === toastId) {
        currentToast = null;
        listeners.forEach(fn => fn());
      }
    }, durationMs);
  }
}

export function Toast() {
  const [toast, setToast] = useState<ToastMessage | null>(null);

  useEffect(() => {
    const update = () => setToast(currentToast ? { ...currentToast } : null);
    listeners.add(update);
    update();
    return () => { listeners.delete(update); };
  }, []);

  if (!toast) return null;

  const bg = toast.type === 'error' ? 'rgba(220,53,69,0.92)' :
             toast.type === 'info' ? 'rgba(13,110,253,0.92)' :
             'rgba(34,139,34,0.92)';

  return (
    <div style={{
      position: 'fixed',
      top: 16,
      left: '50%',
      transform: 'translateX(-50%)',
      zIndex: 10000,
      background: bg,
      color: '#fff',
      padding: '10px 24px',
      borderRadius: 10,
      fontWeight: 600,
      fontSize: 14,
      boxShadow: '0 4px 16px rgba(0,0,0,0.22)',
      pointerEvents: 'none',
      transition: 'opacity 0.25s',
    }}>
      {toast.text}
    </div>
  );
}
