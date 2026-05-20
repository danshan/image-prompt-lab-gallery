export function displayDate(value: string) {
  if (/^\d+$/.test(value)) {
    return new Date(Number(value)).toLocaleString();
  }
  return value;
}

export function formatBytes(value: number | null) {
  if (!value) {
    return "-";
  }
  if (value >= 1024 * 1024 * 1024) {
    return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
  }
  if (value > 1024 * 1024) {
    return `${(value / 1024 / 1024).toFixed(1)} MB`;
  }
  return `${Math.round(value / 1024)} KB`;
}
