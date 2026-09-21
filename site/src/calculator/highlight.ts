/** 誤りのある範囲で，入力を3つに分ける．範囲は，UTF-16ではなく，文字(コードポイント)の番号である． */
interface Split {
  before: string;
  hit: string;
  after: string;
}

function splitAt(source: string, start: number, end: number): Split {
  const chars = [...source];
  return {
    before: chars.slice(0, start).join(''),
    hit: chars.slice(start, end).join(''),
    after: chars.slice(end).join(''),
  };
}

export { splitAt };
export type { Split };
