import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import {
  WebTransportClient,
  WebTransportReceiveStream,
  WebTransportSendStream,
} from '../../wasm-webtransport';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { certificateArrayArray } from './certificate';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, CommonModule, FormsModule],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
})
export class AppComponent {
  public webTransportClient: WebTransportClient = new WebTransportClient();
  public streams: WebTransportSendStream[] = [];
  public message: string = '';
  public selectedStream = 0;

  constructor() {
    this.initSession();
  }

  public async restartWebTransportClient() {
    this.webTransportClient = new WebTransportClient();
  }

  public async initSession(): Promise<void> {
    await this.webTransportClient.init_session(
      'https://localhost:3030',
      new Uint8Array(certificateArrayArray),
    );
  }

  public async startBistreams(): Promise<void> {
    const firstSendStream = await this.webTransportClient.setup_bistream(true);
    this.streams.push(firstSendStream);
    const secondSendStream =
      await this.webTransportClient.setup_bistream(false);
    this.streams.push(secondSendStream);
    this.streams = this.streams.slice();
    await this.webTransportClient.start_bistreams();
  }

  public async sendMessageToStream(
    index: number,
    message: string,
  ): Promise<void> {
    const sendStream = this.streams[index];
    if (sendStream) {
      await sendStream.send_message(message);
    }
  }
}
