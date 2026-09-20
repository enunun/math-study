import { Worker } from 'node:worker_threads';

import type { Element } from 'hast';
import { fromHtml } from 'hast-util-from-html';
import { visit } from 'unist-util-visit';

import type { Environments, Macros } from './macros';
import type { Request, Response } from './worker';

interface RendererOptions {
  macros: Macros;
  environments: Environments;
  /** CHTMLの出力が参照するフォントの，配信先のURL(baseパスを含む)． */
  fontUrl: string;
}

interface Renderer {
  /** TeXの式を，読み上げ用の`aria-label`を付けたhastの要素にする．失敗したときは例外を投げる． */
  render: (tex: string, display: boolean) => Promise<Element>;
  /** これまでに描画したすべての式に必要なCSS． */
  stylesheet: () => Promise<string>;
  /** Workerを終了する．MathJaxの作業スレッドも，これで終了する． */
  shutdown: () => Promise<void>;
}

/** unionの各要素から，同じキーを取り除く． */
type DistributiveOmit<T, K extends PropertyKey> = T extends unknown ? Omit<T, K> : never;

interface WorkerClient {
  send: (request: DistributiveOmit<Request, 'id'>) => Promise<string>;
  shutdown: () => Promise<void>;
}

interface Pending {
  resolve: (value: string) => void;
  reject: (error: Error) => void;
}

/** explorerだけが使う意味づけの属性．読み上げ文字列を`aria-label`に移した後は，不要になる． */
const SEMANTIC_ATTRIBUTE = /^data(?:Semantic|Speech|Braille|Latex)/u;

/** 描画結果から，読み上げ用の属性以外の意味づけを削り，`aria-label`と`role`を付ける． */
function enhance(container: Element): Element {
  const speech = container.properties.dataSemanticSpeechNone;
  visit(container, 'element', (element) => {
    element.properties = Object.fromEntries(
      Object.entries(element.properties).filter(([name]) => !SEMANTIC_ATTRIBUTE.test(name)),
    );
  });
  if (typeof speech === 'string' && speech !== '') {
    container.properties.role = 'math';
    container.properties.ariaLabel = speech;
  }
  return container;
}

/** Workerへリクエストを送り，対応するレスポンスを待つ． */
function createWorkerClient(options: RendererOptions): WorkerClient {
  const pending = new Map<number, Pending>();
  const state: { worker?: Worker; nextId: number } = { nextId: 0 };

  const failAll = (error: Error): void => {
    for (const { reject } of pending.values()) {
      reject(error);
    }
    pending.clear();
  };

  const getWorker = (): Worker => {
    if (state.worker !== undefined) {
      return state.worker;
    }
    const worker = new Worker(new URL('worker.ts', import.meta.url), { workerData: options });
    worker.on('message', (response: Response) => {
      const entry = pending.get(response.id);
      pending.delete(response.id);
      if (response.ok) {
        entry?.resolve(response.value);
      } else {
        entry?.reject(new Error(response.error));
      }
    });
    worker.on('error', failAll);
    state.worker = worker;
    return worker;
  };

  return {
    send: (request) =>
      new Promise<string>((resolve, reject) => {
        state.nextId += 1;
        pending.set(state.nextId, { resolve, reject });
        getWorker().postMessage({ ...request, id: state.nextId });
      }),
    shutdown: async () => {
      await state.worker?.terminate();
      delete state.worker;
      failAll(new Error('Workerが終了した．'));
    },
  };
}

async function runAfter<T>(previous: Promise<unknown>, task: () => Promise<T>): Promise<T> {
  await previous;
  return task();
}

function createRenderer(options: RendererOptions): Renderer {
  const client = createWorkerClient(options);
  // MathJaxは，複数の式を同時に描画する使い方を想定していない．リクエストは，1つずつ処理する．
  const state: { queue: Promise<unknown> } = { queue: Promise.resolve() };

  const enqueue = (request: DistributiveOmit<Request, 'id'>): Promise<string> => {
    const run = runAfter(state.queue, () => client.send(request));
    state.queue = Promise.allSettled([run]);
    return run;
  };

  return {
    async render(tex, display) {
      const html = await enqueue({ type: 'render', tex, display });
      const [container] = fromHtml(html, { fragment: true }).children;
      if (container?.type !== 'element') {
        throw new Error('MathJaxの出力に，要素がありません．');
      }
      return enhance(container);
    },
    stylesheet: () => enqueue({ type: 'stylesheet' }),
    shutdown: client.shutdown,
  };
}

/** プロセスの中で，Workerは1つだけ起動する．最初に指定された設定を使う． */
const shared: { renderer?: Renderer } = {};

function getRenderer(options: RendererOptions): Renderer {
  shared.renderer ??= createRenderer(options);
  return shared.renderer;
}

async function shutdownRenderer(): Promise<void> {
  await shared.renderer?.shutdown();
  delete shared.renderer;
}

export { getRenderer, shutdownRenderer };
export type { Renderer, RendererOptions };
