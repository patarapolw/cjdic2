import { createElement as h } from "react";

export default function Glossary({
  glossary,
  maxLength,
  onTermClicked,
}: {
  glossary: any;
  maxLength?: number;
  onTermClicked: (t: string) => void;
}) {
  function trimString(s: any) {
    if (!maxLength) return s;
    if (typeof s === "string") {
      return s.substring(0, maxLength);
    }
    return s;
  }

  // Plain string
  if (typeof glossary === "string") {
    return trimString(glossary);
  }

  // GlossaryText
  if ("type" in glossary && glossary.type === "text") {
    return trimString(glossary.text);
  }

  // GlossaryImage
  if ("type" in glossary && glossary.type === "image") {
    return <GlossaryImage {...glossary} />;
  }

  // GlossaryStructuredContent
  if ("type" in glossary && glossary.type === "structured-content") {
    const sc = glossary;
    return (
      <StructuredContent
        node={sc.content}
        maxLength={maxLength}
        onTermClicked={onTermClicked}
      />
    );
  }

  // GlossaryDeinflection
  if (Array.isArray(glossary) && glossary.length === 2) {
    const [term, rules] = glossary;
    return <small>{`${term} (${rules.join(", ")})`}</small>;
  }

  return null;
}

function GlossaryImage(img: any) {
  return (
    <img
      src={img.path}
      width={img.width}
      height={img.height}
      title={img.title}
      alt={img.alt}
      style={{
        imageRendering: img.imageRendering || "auto",
        width: `${img.width}em`,
        height: `${img.height}em`,
      }}
    />
  );
}

function StructuredContent({
  node,
  maxLength,
  onTermClicked,
}: {
  node: any;
  maxLength?: number;
  onTermClicked: (t: string) => void;
}): any {
  if (!node) return null;

  function trimString(s: any) {
    if (!maxLength) return s;
    if (typeof s === "string") {
      return s.substring(0, maxLength);
    }
    return s;
  }

  if (typeof node === "string") {
    return trimString(node);
  }

  if (Array.isArray(node)) {
    return (
      <>
        {...node.map((n, i) => (
          <StructuredContent
            key={i}
            node={n}
            maxLength={maxLength}
            onTermClicked={onTermClicked}
          />
        ))}
      </>
    );
  }

  const tag = "tag" in node ? node.tag : null;
  if (!tag) return null;

  // Line break
  if (tag === "br") {
    return <br />;
  }

  // Image
  if (tag === "img") {
    return <GlossaryImage {...node} />;
  }

  // Link
  if (tag === "a") {
    const link = node as any;
    const { href, content } = link;

    const isWebLink = /^https?:\/\//.test(href);

    return (
      <a
        href={href}
        target={isWebLink ? "_blank" : ""}
        onClick={(ev) => {
          if (isWebLink) return true;
          ev.preventDefault();
          const [, t] = href.split("query=");
          if (t) {
            onTermClicked(t.split("&")[0]);
          }
          return false;
        }}
      >
        {content ? (
          <StructuredContent
            node={content}
            maxLength={maxLength}
            onTermClicked={onTermClicked}
          />
        ) : null}
      </a>
    );
  }

  // Block/container elements (span, div, ol, ul, li, ruby, table, etc.)
  const block = node as any;

  return h(
    tag,
    { style: block.style, title: block.title, open: block.open },
    StructuredContent({ node: block.content, maxLength, onTermClicked }),
  );
}
