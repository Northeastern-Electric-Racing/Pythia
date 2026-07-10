import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { CanMessage, NewCanMessageInput } from '../models/can.models';

/** Thin wrapper over the backend's `/messages` endpoints. */
@Injectable({ providedIn: 'root' })
export class MessageApiService {
  private readonly http = inject(HttpClient);

  listMessages(profile: string): Observable<CanMessage[]> {
    return this.http.get<CanMessage[]>('/messages', {
      params: new HttpParams().set('profile', profile),
    });
  }

  createMessage(profile: string, input: NewCanMessageInput): Observable<CanMessage> {
    return this.http.post<CanMessage>('/messages', input, {
      params: new HttpParams().set('profile', profile),
    });
  }

  deleteMessage(id: number): Observable<void> {
    return this.http.delete<void>('/messages', {
      params: new HttpParams().set('id', String(id)),
    });
  }
}
