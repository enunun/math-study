import type { Element, ElementContent } from 'hast';
import { createElement } from 'react';
import type { CSSProperties, ReactElement, ReactNode } from 'react';

/** hastの属性名のうち，Reactでは名前が違うもの． */
const RENAMED: Record<string, string> = {
  ariaHidden: 'aria-hidden',
  ariaLabel: 'aria-label',
};

/** `margin-top`のようなCSSの名前を，`marginTop`にする． */
function camelCase(name: string): string {
  return name
    .split('-')
    .map((part, index) => (index === 0 ? part : `${part.charAt(0).toUpperCase()}${part.slice(1)}`))
    .join('');
}

/** `left: 5%; top: 10%`のような文字列を，Reactのstyleの値にする．`--`で始まる名前は，そのまま残す． */
function parseStyle(text: string): CSSProperties {
  const style: Record<string, string> = {};
  for (const declaration of text.split(';')) {
    const colon = declaration.indexOf(':');
    if (colon > 0) {
      const name = declaration.slice(0, colon).trim();
      style[name.startsWith('--') ? name : camelCase(name)] = declaration.slice(colon + 1).trim();
    }
  }
  return style;
}

/** hastの属性を，Reactの属性にする．`className`は，配列を空白でつなぐ． */
function toProps(element: Element, key: string): Record<string, unknown> {
  const props: Record<string, unknown> = { key };
  for (const [name, value] of Object.entries(element.properties)) {
    if (name === 'className' && Array.isArray(value)) {
      props.className = value.join(' ');
    } else if (name === 'style' && typeof value === 'string') {
      props.style = parseStyle(value);
    } else {
      props[RENAMED[name] ?? name] = value;
    }
  }
  return props;
}

/** hastの木を，Reactの要素にする．`render`が要素を返したときは，それに置き換える．偽を返したときは，そのまま変換する． */
function hastToReact(
  node: ElementContent,
  key: string,
  render: (element: Element, key: string) => ReactElement | false = () => false,
): ReactNode {
  if (node.type === 'text') {
    return node.value;
  }
  if (node.type !== 'element') {
    return null;
  }
  const replaced = render(node, key);
  if (replaced !== false) {
    return replaced;
  }
  return createElement(
    node.tagName,
    toProps(node, key),
    ...node.children.map((child, index) => hastToReact(child, `${key}.${index}`, render)),
  );
}

export { hastToReact };
