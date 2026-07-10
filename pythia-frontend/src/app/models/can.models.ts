/** Transmission mode of a CAN frame, matching the backend's `mode` column. */
export type CanMode = 'oneshot' | 'broadcast';

/** Frame format. Maps to the backend's integer `is_extended` (std=0, ext=1). */
export type CanFormat = 'std' | 'ext';

/** A named collection of CAN frames. */
export interface TestProfile {
  id: number;
  name: string;
}

/** A CAN frame as returned by the backend (`GET /messages`). */
export interface CanMessage {
  id: number;
  profile_id: number;
  can_id: number;
  is_extended: number; // 0 | 1
  data: number[]; // 0-8 bytes, each 0-255
  mode: CanMode;
  offset_ms: number;
  period_ms: number | null;
}

/** Body for `POST /messages` — no id (DB-assigned) or profile_id (from query param). */
export interface NewCanMessageInput {
  can_id: number;
  is_extended: number; // 0 | 1
  data: number[];
  mode: CanMode;
  offset_ms: number;
  period_ms: number | null;
}
