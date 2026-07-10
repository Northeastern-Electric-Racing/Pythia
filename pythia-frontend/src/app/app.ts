import { Component, OnInit, effect, inject } from '@angular/core';
import { MatSnackBar } from '@angular/material/snack-bar';
import { ShellComponent } from './components/shell/shell';
import { ProfileSelectorComponent } from './components/profile-selector/profile-selector';
import { CanFrameTableComponent } from './components/can-frame-table/can-frame-table';
import { AddFrameFormComponent } from './components/add-frame-form/add-frame-form';
import { ProfileStore } from './state/profile.store';

@Component({
  selector: 'app-root',
  imports: [ShellComponent, ProfileSelectorComponent, CanFrameTableComponent, AddFrameFormComponent],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App implements OnInit {
  private readonly store = inject(ProfileStore);
  private readonly snackBar = inject(MatSnackBar);

  constructor() {
    effect(() => {
      const err = this.store.error();
      if (err) {
        this.snackBar.open(err, 'Dismiss', { duration: 5000 });
        this.store.clearError();
      }
    });
  }

  ngOnInit(): void {
    this.store.loadProfiles();
  }
}
