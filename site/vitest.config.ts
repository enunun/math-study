import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
    // MathJaxの起動に，数秒かかる．
    testTimeout: 30_000,
  },
});
