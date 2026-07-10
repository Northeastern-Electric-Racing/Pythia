import { Component, inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatChipsModule } from '@angular/material/chips';
import { MatIconModule } from '@angular/material/icon';
import { MatTableModule } from '@angular/material/table';
import { CanMessage } from '../../models/can.models';
import { bytesToNumber, formatCanId } from '../../utils/payload.util';
import { ProfileStore } from '../../state/profile.store';

/** Read-only table of the selected profile's CAN frames, with per-row delete. */
@Component({
  selector: 'app-can-frame-table',
  imports: [MatTableModule, MatChipsModule, MatIconModule, MatButtonModule],
  templateUrl: './can-frame-table.html',
  styleUrl: './can-frame-table.scss',
})
export class CanFrameTableComponent {
  readonly store = inject(ProfileStore);
  readonly columns = ['canId', 'format', 'content', 'mode', 'tStart', 'period', 'actions'];

  canId(frame: CanMessage): string {
    return formatCanId(frame.can_id);
  }

  content(frame: CanMessage): string {
    return bytesToNumber(frame.data).toString();
  }

  delete(frame: CanMessage): void {
    if (confirm(`Delete frame ${this.canId(frame)}?`)) {
      this.store.deleteFrame(frame.id);
    }
  }
}
