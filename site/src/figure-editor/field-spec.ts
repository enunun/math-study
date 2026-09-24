import type { Option } from './field-options';

/** 項目の入力欄の種類と，内容． */
type FieldSpec =
  | { kind: 'text'; key: string; label: string; optional?: boolean }
  | { kind: 'number'; key: string; label: string; optional?: boolean }
  | { kind: 'bound'; key: string; label: string; optional?: boolean }
  /** 数か式を並べる．`count`が`'dimension'`なら，平面で2個，空間で3個である． */
  | {
      kind: 'list';
      key: string;
      label: string;
      item: 'text' | 'number' | 'bound';
      count: number | 'dimension';
      optional?: boolean;
    }
  /** 2つの変数の範囲(`[[下端, 上端], [下端, 上端]]`)． */
  | { kind: 'domain2'; key: string; label: string }
  | { kind: 'select'; key: string; label: string; options: readonly Option[]; optional?: boolean }
  | { kind: 'checkbox'; key: string; label: string; initial: boolean }
  /** 座標(数か式の並び)か，点の式(`A + B`など)． */
  | { kind: 'position'; key: string; label: string }
  /** 別のオブジェクトの識別子． */
  | { kind: 'reference'; key: string; label: string; of: readonly string[] }
  | { kind: 'references'; key: string; label: string; of: readonly string[]; count: number }
  /** JSONの値を，そのまま書く． */
  | { kind: 'json'; key: string; label: string; optional?: boolean; hint: string }
  /** 変換(`transform`)．手順の並びをJSONで書き，手順のテンプレートを選んで足せる． */
  | { kind: 'transform'; key: string; label: string }
  | { kind: 'style' }
  /** あれば描く，スタイルつきの項目(曲面のワイヤーフレームなど)．チェックボックスで有無を選ぶ． */
  | { kind: 'toggleStyle'; key: string; label: string };

export type { FieldSpec };
