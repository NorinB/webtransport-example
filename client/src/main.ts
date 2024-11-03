import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { AppComponent } from './app/app.component';
import { initWasm } from '../wasm-webtransport';

const startApp = async () => {
  await initWasm();

  bootstrapApplication(AppComponent, appConfig).catch((err) =>
    console.error(err),
  );
};

startApp();
