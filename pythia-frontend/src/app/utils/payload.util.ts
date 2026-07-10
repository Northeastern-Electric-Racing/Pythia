/** Largest payload value that fits in 8 bytes. */
export const MAX_PAYLOAD = 2n ** 64n - 1n;

/** Largest CAN id for a standard (11-bit) frame. */
export const MAX_STD_ID = 0x7ff;
/** Largest CAN id for an extended (29-bit) frame. */
export const MAX_EXT_ID = 0x1fffffff;

/**
 * Convert a non-negative payload value into a big-endian byte array of minimal
 * length (0-8 bytes). `0n` yields `[]`. Throws if negative or > 2^64-1.
 */
export function numberToBytes(value: bigint): number[] {
  if (value < 0n) throw new Error('payload must be non-negative');
  if (value > MAX_PAYLOAD) throw new Error('payload exceeds 8 bytes');
  const bytes: number[] = [];
  let v = value;
  while (v > 0n) {
    bytes.unshift(Number(v & 0xffn));
    v >>= 8n;
  }
  return bytes;
}

/** Inverse of {@link numberToBytes}: big-endian byte array to a bigint. */
export function bytesToNumber(bytes: number[]): bigint {
  return bytes.reduce((acc, b) => (acc << 8n) | BigInt(b & 0xff), 0n);
}

/**
 * Parse a user-entered decimal string into a payload bigint.
 * Throws on empty/non-numeric input.
 */
export function parsePayload(input: string): bigint {
  const trimmed = input.trim();
  if (!/^\d+$/.test(trimmed)) throw new Error('payload must be a whole number');
  return BigInt(trimmed);
}

/**
 * Parse a user-entered CAN id (hex, with or without `0x` prefix) into a number.
 * Throws on non-hex input.
 */
export function parseCanId(input: string): number {
  const trimmed = input.trim().replace(/^0x/i, '');
  if (!/^[0-9a-f]+$/i.test(trimmed)) throw new Error('CAN id must be hex');
  return parseInt(trimmed, 16);
}

/** Format a CAN id as an uppercase hex string with `0x` prefix. */
export function formatCanId(id: number): string {
  return '0x' + id.toString(16).toUpperCase();
}
