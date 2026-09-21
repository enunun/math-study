import { defineConfig } from 'vitest/config';

export default defineConfig({
  resolve: {
    // tsconfigの別名(@/)と同じ．
    alias: { '@': `${import.meta.dirname}/src` },
  },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
    // MathJaxの起動に，数秒かかる．
    testTimeout: 30_000,
  },
});
