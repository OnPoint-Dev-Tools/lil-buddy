export function TextShimmer(props: { text: string; className?: string }) {
  return (
    <span className={props.className ? `lm-text-shimmer ${props.className}` : 'lm-text-shimmer'}>
      {props.text}
    </span>
  );
}
