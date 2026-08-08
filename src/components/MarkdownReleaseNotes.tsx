import ReactMarkdown from "react-markdown";

interface MarkdownReleaseNotesProps {
  body: string;
  className?: string;
}

const TRUSTED_PREFIX = "https://github.com/Eeymoo/peregrine/releases/download/";

export function MarkdownReleaseNotes({ body, className }: MarkdownReleaseNotesProps) {
  return (
    <div
      className={
        "prose prose-sm dark:prose-invert text-muted-foreground " +
        (className ?? "")
      }
    >
      <ReactMarkdown
        components={{
          a: ({ node, href, children, ...props }) => {
            const isTrusted =
              typeof href === "string" && href.startsWith(TRUSTED_PREFIX);
            return isTrusted ? (
              <a {...props} href={href} target="_blank" rel="noopener noreferrer">
                {children}
              </a>
            ) : (
              <span {...props} className="text-muted-foreground">
                {children}
              </span>
            );
          },
        }}
      >
        {body}
      </ReactMarkdown>
    </div>
  );
}
