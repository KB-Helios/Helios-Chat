import {
  ActivityIcon,
  BotMessageSquareIcon,
  BoxesIcon,
  CompassIcon,
  Settings2Icon,
  SparklesIcon,
} from "lucide-react"
import type { ComponentProps, ComponentType } from "react"

import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

export type AppView = "chat" | "discover" | "models" | "settings" | "diagnostics"

type NavItem = {
  icon: ComponentType<{ className?: string }>
  label: string
  view: AppView
}

const navItems: NavItem[] = [
  { icon: BotMessageSquareIcon, label: "Chat", view: "chat" },
  { icon: CompassIcon, label: "Discover", view: "discover" },
  { icon: BoxesIcon, label: "Models", view: "models" },
  { icon: Settings2Icon, label: "Settings", view: "settings" },
  { icon: ActivityIcon, label: "Diagnostics", view: "diagnostics" },
]

export function AppSidebar({
  activeView,
  onViewChange,
  ...props
}: ComponentProps<typeof Sidebar> & {
  activeView: AppView
  onViewChange(view: AppView): void
}) {
  return (
    <Sidebar collapsible="offcanvas" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton className="data-[slot=sidebar-menu-button]:p-1.5!">
              <SparklesIcon className="size-5!" />
              <span className="text-base font-semibold">Helios Chat</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <SidebarMenu>
          {navItems.map((item) => (
            <SidebarMenuItem key={item.view}>
              <SidebarMenuButton
                isActive={activeView === item.view}
                onClick={() => onViewChange(item.view)}
              >
                <item.icon className="size-4" />
                <span>{item.label}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ))}
        </SidebarMenu>
      </SidebarContent>
    </Sidebar>
  )
}
