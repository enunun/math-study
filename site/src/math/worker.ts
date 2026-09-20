import { parentPort, workerData } from 'node:worker_threads';

/**
 * MathJaxを動かすWorkerスレッド．ViteやAstroの実行環境を通さず，Node.jsのネイティブなモジュールとして動く．
 * メインスレッドとは，メッセージでやり取りする．
 */

interface WorkerInit {
  macros: Record<string, string | [string, number]>;
  fontUrl: string;
}

type Request =
  | { id: number; type: 'render'; tex: string; display: boolean }
  | { id: number; type: 'stylesheet' };

type Response = { id: number; ok: true; value: string } | { id: number; ok: false; error: string };

interface MathJaxGlobal {
  startup: {
    promise: Promise<unknown>;
    adaptor: { outerHTML: (node: unknown) => string; textContent: (node: unknown) => string };
  };
  tex2chtmlPromise: (tex: string, options: Record<string, unknown>) => Promise<unknown>;
  chtmlStylesheet: () => unknown;
}

const EM = 16;
const EX = 8;
const CONTAINER_WIDTH = 1280;

/**
 * 未定義のマクロや構文の誤りを，描画の失敗にする．
 * MathJaxの既定では，未定義のマクロはそのまま文字として描画されてしまう．
 */
function buildConfig(init: WorkerInit): Record<string, unknown> {
  return {
    loader: {
      paths: { mathjax: '@mathjax/src/bundle' },
      load: ['adaptors/liteDOM', '[tex]/bussproofs'],
      require: (file: string): Promise<unknown> => import(file),
    },
    tex: {
      packages: { '[-]': ['noundefined'], '[+]': ['bussproofs'] },
      macros: init.macros,
      formatError: (_jax: unknown, error: Error) => {
        throw error;
      },
    },
    // 幅より長い別行立ての式は，画面の外へはみ出さず，その式の中でスクロールさせる．
    chtml: { fontURL: init.fontUrl, displayOverflow: 'scroll' },
    options: { sre: { locale: 'en' } },
  };
}

function getMathJax(): MathJaxGlobal {
  return Reflect.get(globalThis, 'MathJax') as MathJaxGlobal;
}

/** リクエストを処理して，結果の文字列を返す． */
async function handle(request: Request): Promise<string> {
  const mathJax = getMathJax();
  const { adaptor } = mathJax.startup;
  if (request.type === 'stylesheet') {
    // これまでに描画したすべての式で使った文字の分を含む．
    return adaptor.textContent(mathJax.chtmlStylesheet());
  }
  const node = await mathJax.tex2chtmlPromise(request.tex, {
    display: request.display,
    em: EM,
    ex: EX,
    containerWidth: CONTAINER_WIDTH,
  });
  return adaptor.outerHTML(node);
}

/** 例外からメッセージを取り出す．MathJaxのTexErrorは，Errorを継承しないため，messageの有無で判断する． */
function messageOf(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    return String(error.message);
  }
  return String(error);
}

/** リクエストを処理し，成功と失敗のどちらも，レスポンスとして返す． */
async function respond(request: Request): Promise<Response> {
  try {
    return { id: request.id, ok: true, value: await handle(request) };
  } catch (error) {
    return { id: request.id, ok: false, error: messageOf(error) };
  }
}

Reflect.set(globalThis, 'MathJax', buildConfig(workerData as WorkerInit));
await import('@mathjax/src/bundle/tex-chtml.js');
await getMathJax().startup.promise;

parentPort?.on('message', async (request: Request) => {
  parentPort?.postMessage(await respond(request));
});

export type { Request, Response, WorkerInit };
