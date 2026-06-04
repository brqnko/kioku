import type { ComponentChildren } from "preact";

export interface BreadcrumbItem {
  label: ComponentChildren;
  href?: string;
}

interface PageHeaderProps {
  title: ComponentChildren;
  description?: ComponentChildren;
  breadcrumbs?: BreadcrumbItem[];
  actions?: ComponentChildren;
}

export function PageHeader({
  title,
  description,
  breadcrumbs = [],
  actions,
}: PageHeaderProps) {
  return (
    <section class="page-header">
      <div class="min-w-0 flex flex-col gap-3">
        {breadcrumbs.length > 0 && (
          <nav class="breadcrumb" aria-label="Breadcrumb">
            {breadcrumbs.map((item, index) => {
              const current = index === breadcrumbs.length - 1;
              return (
                <span key={index} class="breadcrumb-item">
                  {index > 0 && (
                    <span class="material-symbols-outlined text-[16px] text-text-disabled">
                      chevron_right
                    </span>
                  )}
                  {item.href && !current ? (
                    <a
                      href={item.href}
                      class="hover:text-text-primary no-underline text-inherit truncate"
                    >
                      {item.label}
                    </a>
                  ) : (
                    <span class="text-text-primary truncate">{item.label}</span>
                  )}
                </span>
              );
            })}
          </nav>
        )}
        <div class="flex flex-col gap-2">
          <h1 class="heading-h2 truncate">{title}</h1>
          {description && (
            <p class="text-body text-text-secondary max-w-3xl">
              {description}
            </p>
          )}
        </div>
      </div>
      {actions && (
        <div class="flex items-center gap-2 tablet:gap-3 flex-wrap">
          {actions}
        </div>
      )}
    </section>
  );
}
