import { Component, OnInit, inject } from '@angular/core';
import { ShellComponent } from './components/shell/shell';
import { ProfileSelectorComponent } from './components/profile-selector/profile-selector';
import { ProfileStore } from './state/profile.store';

@Component({
  selector: 'app-root',
  imports: [ShellComponent, ProfileSelectorComponent],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App implements OnInit {
  private readonly store = inject(ProfileStore);
  ngOnInit(): void {
    this.store.loadProfiles();
  }
}
