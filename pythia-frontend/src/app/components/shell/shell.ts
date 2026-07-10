import { Component } from '@angular/core';
import { MatTabsModule } from '@angular/material/tabs';
import { MatIconModule } from '@angular/material/icon';

/** Branded app frame: header, tab bar, and a content area for the CAN config. */
@Component({
  selector: 'app-shell',
  imports: [MatTabsModule, MatIconModule],
  templateUrl: './shell.html',
  styleUrl: './shell.scss',
})
export class ShellComponent {}
