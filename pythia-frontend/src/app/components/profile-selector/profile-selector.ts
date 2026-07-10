import { Component, inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { NewProfileDialogComponent } from '../new-profile-dialog/new-profile-dialog';
import { ProfileStore } from '../../state/profile.store';

/** Profile dropdown plus create/delete actions. */
@Component({
  selector: 'app-profile-selector',
  imports: [MatFormFieldModule, MatSelectModule, MatButtonModule, MatIconModule],
  templateUrl: './profile-selector.html',
  styleUrl: './profile-selector.scss',
})
export class ProfileSelectorComponent {
  readonly store = inject(ProfileStore);
  private readonly dialog = inject(MatDialog);

  onSelect(name: string): void {
    this.store.selectProfile(name);
  }

  openCreate(): void {
    this.dialog.open(NewProfileDialogComponent);
  }

  deleteSelected(): void {
    const name = this.store.selectedProfile();
    if (name && confirm(`Delete profile "${name}" and all its frames?`)) {
      this.store.deleteProfile(name);
    }
  }
}
