import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { HttpErrorResponse } from '@angular/common/http';
import { MatButtonModule } from '@angular/material/button';
import { MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { ProfileStore } from '../../state/profile.store';

/** Dialog for creating a new test profile. Resolves to the created name or null. */
@Component({
  selector: 'app-new-profile-dialog',
  imports: [FormsModule, MatDialogModule, MatFormFieldModule, MatInputModule, MatButtonModule],
  templateUrl: './new-profile-dialog.html',
  styleUrl: './new-profile-dialog.scss',
})
export class NewProfileDialogComponent {
  private readonly store = inject(ProfileStore);
  private readonly dialogRef = inject(MatDialogRef<NewProfileDialogComponent>);

  readonly name = signal('');
  readonly error = signal<string | null>(null);
  readonly saving = signal(false);

  submit(): void {
    const trimmed = this.name().trim();
    if (!trimmed) {
      this.error.set('Name is required');
      return;
    }
    this.saving.set(true);
    this.error.set(null);
    this.store.createProfile(trimmed).subscribe({
      next: () => this.dialogRef.close(trimmed),
      error: (err: HttpErrorResponse) => {
        this.saving.set(false);
        this.error.set(
          err.status === 409
            ? 'A profile with that name already exists'
            : 'Failed to create profile',
        );
      },
    });
  }

  cancel(): void {
    this.dialogRef.close(null);
  }
}
