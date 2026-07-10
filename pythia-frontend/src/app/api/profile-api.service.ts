import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { TestProfile } from '../models/can.models';

/** Thin wrapper over the backend's `/profiles` endpoints. */
@Injectable({ providedIn: 'root' })
export class ProfileApiService {
  private readonly http = inject(HttpClient);

  listProfiles(): Observable<string[]> {
    return this.http.get<string[]>('/profiles');
  }

  createProfile(name: string): Observable<TestProfile> {
    return this.http.post<TestProfile>('/profiles', { name });
  }

  deleteProfile(name: string): Observable<void> {
    return this.http.delete<void>('/profiles', {
      params: new HttpParams().set('name', name),
    });
  }
}
