import type { VariantProps } from "class-variance-authority";
import { cva } from "class-variance-authority";

export { default as Toggle } from "./Toggle.vue";

export const toggleVariants = cva(
  "inline-flex items-center justify-center gap-1 whitespace-nowrap rounded-md font-medium text-sm outline-none transition-[color,box-shadow] hover:bg-muted hover:text-muted-foreground focus-visible:border-ring focus-visible:ring-[1px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-70 aria-invalid:border-destructive aria-invalid:ring-destructive/20 data-[state=on]:bg-accent/10 data-[state=on]:text-accent dark:aria-invalid:ring-destructive/40 [&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default:
          "rounded-full border border-input hover:bg-accent/5 data-[state=on]:bg-accent/5 data-[state=on]:text-accent",
        outline:
          "border border-input bg-transparent shadow-xs hover:bg-accent hover:text-accent-foreground",
      },
      size: {
        default: "h-7 min-w-8 px-2 md:h-8 md:px-2.5",
        sm: "h-9 min-w-9 px-2",
        xs: "h-8 min-w-8 px-1.5",
        lg: "h-10 min-w-10 px-2.5",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export type ToggleVariants = VariantProps<typeof toggleVariants>;
