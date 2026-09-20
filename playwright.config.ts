import { defineConfig } from '@playwright/test';

const PORT = 4322;
const BASE_URL = `http://127.0.0.1:${PORT}/math-study/`;

// ビルド済みのサイト(site/dist)を，公開時と同じbaseパスで配信して確認する．
export default defineConfig({
  testDir: 'e2e',
  testMatch: '**/*.spec.ts',
  fullyParallel: true,
  reporter: 'list',
  use: { baseURL: BASE_URL },
  webServer: {
    command: 'node e2e/serve.ts',
    url: BASE_URL,
    reuseExistingServer: true,
  },
});
