import { useState } from 'react';
import type { ReactElement } from 'react';

/** 選べる項目． */
interface PickerItem {
  id: string;
  label: string;
}

/** 項目のまとまり．項目の選択欄では，まとまりごとに見出し(`optgroup`)を付ける． */
interface PickerGroup<T extends PickerItem> {
  label: string;
  items: readonly T[];
}

/** 分類．分類を選ぶと，項目の選択欄に，その分類のまとまりだけが出る． */
interface PickerCategory<T extends PickerItem> {
  label: string;
  groups: readonly PickerGroup<T>[];
}

/** 分類の上の段．選ぶと，分類の選択欄に，その段の分類だけが出る(見本の平面・空間)． */
interface PickerTier<T extends PickerItem> {
  label: string;
  categories: readonly PickerCategory<T>[];
}

interface Props<T extends PickerItem> {
  /** 選択の見出し(「見本」「テンプレート(部品)」など)． */
  title: string;
  /** 項目の選択欄の名前(「見本」「部品」など)．分類の選択欄は「〈名前〉の分類」になる． */
  name: string;
  categories: readonly PickerCategory<T>[];
  /** ボタンの文字(「読み込む」「挿入」など)． */
  action: string;
  onPick: (item: T) => void;
  /** 分類の選択欄の前に置く要素(上の段の選択欄)． */
  leading?: ReactElement;
}

/** 項目の選択欄の値．まとまりが違えば，項目の識別子が同じでも区別する． */
function keyOf(groupIndex: number, item: PickerItem): string {
  return `${groupIndex}/${item.id}`;
}

function firstKey(category: PickerCategory<PickerItem> | undefined): string {
  const item = category?.groups[0]?.items[0];
  return item === undefined ? '' : keyOf(0, item);
}

function options(groupIndex: number, items: readonly PickerItem[]): ReactElement[] {
  return items.map((item) => (
    <option key={item.id} value={keyOf(groupIndex, item)}>
      {item.label}
    </option>
  ));
}

/** 項目の選択欄．まとまりが2つ以上あれば，まとまりごとに見出しを付ける． */
function ItemSelect({
  name,
  groups,
  value,
  onChange,
}: {
  name: string;
  groups: readonly PickerGroup<PickerItem>[];
  value: string;
  onChange: (value: string) => void;
}): ReactElement {
  return (
    <select
      aria-label={name}
      value={value}
      onChange={(event) => {
        onChange(event.target.value);
      }}
    >
      {groups.length > 1
        ? groups.map((group, groupIndex) => (
            <optgroup key={group.label} label={group.label}>
              {options(groupIndex, group.items)}
            </optgroup>
          ))
        : options(0, groups[0]?.items ?? [])}
    </select>
  );
}

/**
 * 分類，項目の順に選び，ボタンで決める選択．項目が多くなっても，一度に見えるのは1つの分類だけである．
 */
function GroupedPicker<T extends PickerItem>({
  title,
  name,
  categories,
  action,
  onPick,
  leading,
}: Props<T>): ReactElement {
  const [categoryLabel, setCategoryLabel] = useState(categories[0]?.label ?? '');
  const category =
    categories.find((candidate) => candidate.label === categoryLabel) ?? categories[0];
  const groups = category?.groups ?? [];
  const entries = groups.flatMap((group, groupIndex) =>
    group.items.map((item) => [keyOf(groupIndex, item), item] as const),
  );
  const [choice, setChoice] = useState(firstKey(category));
  const current = entries.find(([key]) => key === choice) ?? entries[0];
  return (
    <div className="fe-template-picker" role="group" aria-label={title}>
      <span>{title}：</span>
      {leading}
      <select
        aria-label={`${name}の分類`}
        value={category?.label ?? ''}
        onChange={(event) => {
          const label = event.target.value;
          setCategoryLabel(label);
          setChoice(firstKey(categories.find((candidate) => candidate.label === label)));
        }}
      >
        {categories.map(({ label }) => (
          <option key={label} value={label}>
            {label}
          </option>
        ))}
      </select>
      <ItemSelect name={name} groups={groups} value={current?.[0] ?? ''} onChange={setChoice} />
      <button
        type="button"
        onClick={() => {
          if (current !== undefined) {
            onPick(current[1]);
          }
        }}
      >
        {action}
      </button>
    </div>
  );
}

/**
 * 段，分類，項目の順に選ぶ選択．段を変えると，分類と項目の選択欄を作り直す(`key`に段を含める)．
 * 段の選択欄は「〈名前〉の種類」である．
 */
function TieredPicker<T extends PickerItem>({
  tiers,
  ...props
}: Omit<Props<T>, 'categories' | 'leading'> & {
  tiers: readonly PickerTier<T>[];
}): ReactElement {
  const [tierLabel, setTierLabel] = useState(tiers[0]?.label ?? '');
  const tier = tiers.find((candidate) => candidate.label === tierLabel) ?? tiers[0];
  const leading = (
    <select
      aria-label={`${props.name}の種類`}
      value={tier?.label ?? ''}
      onChange={(event) => {
        setTierLabel(event.target.value);
      }}
    >
      {tiers.map(({ label }) => (
        <option key={label} value={label}>
          {label}
        </option>
      ))}
    </select>
  );
  return (
    <GroupedPicker
      key={tier?.label ?? ''}
      {...props}
      categories={tier?.categories ?? []}
      leading={leading}
    />
  );
}

export { GroupedPicker, TieredPicker };
export type { PickerCategory, PickerGroup, PickerItem, PickerTier };
