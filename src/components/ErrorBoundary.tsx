import { Component, type ErrorInfo, type ReactNode } from 'react';

type Props = {
  children: ReactNode;
};

type State = {
  error: Error | null;
};

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('Lil Buddy UI crashed:', error, info);
  }

  render() {
    if (this.state.error) {
      return (
        <div className="lm-error-boundary">
          <strong>Lil Buddy UI hit an error.</strong>
          <span>{this.state.error.message}</span>
          <button onClick={() => this.setState({ error: null })}>Try again</button>
        </div>
      );
    }

    return this.props.children;
  }
}
