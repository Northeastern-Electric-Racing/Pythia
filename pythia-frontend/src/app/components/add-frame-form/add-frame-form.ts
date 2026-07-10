import { Component, inject, signal } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';
import { FormControl, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { CanFormat, CanMode, NewCanMessageInput } from '../../models/can.models';
import {
  MAX_EXT_ID, MAX_PAYLOAD, MAX_STD_ID, numberToBytes, parseCanId, parsePayload,
} from '../../utils/payload.util';
import { ProfileStore } from '../../state/profile.store';

/** Form to add a CAN frame to the selected profile. */
@Component({
  selector: 'app-add-frame-form',
  imports: [
    ReactiveFormsModule, MatFormFieldModule, MatInputModule,
    MatSelectModule, MatButtonModule, MatIconModule,
  ],
  templateUrl: './add-frame-form.html',
  styleUrl: './add-frame-form.scss',
})
export class AddFrameFormComponent {
  readonly store = inject(ProfileStore);
  readonly submitError = signal<string | null>(null);

  readonly form = new FormGroup({
    canId: new FormControl('', { nonNullable: true, validators: [Validators.required] }),
    format: new FormControl<CanFormat>('std', { nonNullable: true }),
    payload: new FormControl('0', { nonNullable: true, validators: [Validators.required] }),
    mode: new FormControl<CanMode>('oneshot', { nonNullable: true }),
    offsetMs: new FormControl(0, {
      nonNullable: true,
      validators: [Validators.required, Validators.min(0)],
    }),
    periodMs: new FormControl<number | null>(null),
  });

  readonly isBroadcast = signal(false);

  constructor() {
    this.form.controls.mode.valueChanges.subscribe((mode) => {
      const broadcast = mode === 'broadcast';
      this.isBroadcast.set(broadcast);
      const period = this.form.controls.periodMs;
      if (broadcast) {
        period.setValidators([Validators.required, Validators.min(1)]);
      } else {
        period.clearValidators();
        period.setValue(null);
      }
      period.updateValueAndValidity();
    });
  }

  submit(): void {
    this.submitError.set(null);
    if (!this.store.selectedProfile()) {
      this.submitError.set('Select a profile first');
      return;
    }
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const v = this.form.getRawValue();

    let can_id: number;
    let data: number[];
    try {
      can_id = parseCanId(v.canId);
      const payload = parsePayload(v.payload);
      if (payload > MAX_PAYLOAD) throw new Error('Payload exceeds 8 bytes');
      data = numberToBytes(payload);
    } catch (e) {
      this.submitError.set((e as Error).message);
      return;
    }

    const is_extended = v.format === 'ext' ? 1 : 0;
    const maxId = is_extended ? MAX_EXT_ID : MAX_STD_ID;
    if (can_id < 0 || can_id > maxId) {
      this.submitError.set(
        `CAN id out of range for ${v.format.toUpperCase()} (max 0x${maxId.toString(16).toUpperCase()})`,
      );
      return;
    }

    const input: NewCanMessageInput = {
      can_id,
      is_extended,
      data,
      mode: v.mode,
      offset_ms: v.offsetMs,
      period_ms: v.mode === 'broadcast' ? v.periodMs : null,
    };

    this.store.addFrame(input).subscribe({
      next: () => this.reset(),
      error: (err: HttpErrorResponse) => {
        this.submitError.set(
          err.status === 400 ? (err.error ?? 'Invalid frame')
          : err.status === 404 ? 'Profile not found'
          : 'Failed to add frame',
        );
      },
    });
  }

  private reset(): void {
    this.form.reset({ canId: '', format: 'std', payload: '0', mode: 'oneshot', offsetMs: 0, periodMs: null });
    this.isBroadcast.set(false);
  }
}
