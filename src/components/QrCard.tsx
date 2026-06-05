export function QrCard({
  kind,
  src,
  alt,
  text,
}: {
  kind: "qq" | "alipay";
  src: string;
  alt: string;
  text: string;
}) {
  return (
    <div className={`qrCard ${kind}`}>
      <img src={src} alt={alt} />
      <p>{text}</p>
    </div>
  );
}
