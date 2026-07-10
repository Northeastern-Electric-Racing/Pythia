import { Injectable, inject, signal } from '@angular/core';
import { Observable, tap } from 'rxjs';
import { MessageApiService } from '../api/message-api.service';
import { ProfileApiService } from '../api/profile-api.service';
import { CanMessage, NewCanMessageInput, TestProfile } from '../models/can.models';

/**
 * Signal-based facade over the profile/message APIs. Components read the signals
 * and call the methods; mutations re-fetch to stay consistent with the backend.
 */
@Injectable({ providedIn: 'root' })
export class ProfileStore {
  private readonly profileApi = inject(ProfileApiService);
  private readonly messageApi = inject(MessageApiService);

  private readonly _profiles = signal<string[]>([]);
  private readonly _selectedProfile = signal<string | null>(null);
  private readonly _frames = signal<CanMessage[]>([]);
  private readonly _loading = signal(false);
  private readonly _error = signal<string | null>(null);

  readonly profiles = this._profiles.asReadonly();
  readonly selectedProfile = this._selectedProfile.asReadonly();
  readonly frames = this._frames.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  loadProfiles(): void {
    this._loading.set(true);
    this.profileApi.listProfiles().subscribe({
      next: (names) => {
        this._profiles.set(names);
        this._loading.set(false);
      },
      error: () => this.fail('Failed to load profiles'),
    });
  }

  selectProfile(name: string): void {
    this._selectedProfile.set(name);
    this.refreshFrames();
  }

  createProfile(name: string): Observable<TestProfile> {
    return this.profileApi.createProfile(name).pipe(
      tap((profile) => {
        this._profiles.update((list) => [...list, profile.name].sort());
        this.selectProfile(profile.name);
      }),
    );
  }

  addFrame(input: NewCanMessageInput): Observable<CanMessage> {
    const profile = this._selectedProfile();
    if (!profile) throw new Error('no profile selected');
    return this.messageApi.createMessage(profile, input).pipe(
      tap(() => this.refreshFrames()),
    );
  }

  deleteFrame(id: number): void {
    this.messageApi.deleteMessage(id).subscribe({
      next: () => this.refreshFrames(),
      error: () => this.fail('Failed to delete frame'),
    });
  }

  deleteProfile(name: string): void {
    this.profileApi.deleteProfile(name).subscribe({
      next: () => {
        this._profiles.update((list) => list.filter((n) => n !== name));
        if (this._selectedProfile() === name) {
          this._selectedProfile.set(null);
          this._frames.set([]);
        }
      },
      error: () => this.fail('Failed to delete profile'),
    });
  }

  clearError(): void {
    this._error.set(null);
  }

  private refreshFrames(): void {
    const profile = this._selectedProfile();
    if (!profile) {
      this._frames.set([]);
      return;
    }
    this._loading.set(true);
    this.messageApi.listMessages(profile).subscribe({
      next: (frames) => {
        this._frames.set(frames);
        this._loading.set(false);
      },
      error: () => this.fail('Failed to load frames'),
    });
  }

  private fail(message: string): void {
    this._error.set(message);
    this._loading.set(false);
  }
}
