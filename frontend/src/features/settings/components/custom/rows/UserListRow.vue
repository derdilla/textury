<script setup lang="ts" generic="T extends { id: UserId; name: string; subtitle?: string }">
import { ref } from "vue";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import type { UserId } from "@/api/types";

export type UserBadge = {
  label: string;
  class?: string;
};

export type MenuItem = {
  id: string;
  label: string;
  icon?: string;
  isDestructive?: boolean;
  isDivider?: boolean;
  onClick?: () => void;
};

const props = withDefaults(
  defineProps<{
    users: T[];
    isLoading?: boolean;
    showHeader?: boolean;
    title?: string;
    description?: string;
    getBadges?: (user: T) => UserBadge[];
    getMenuItems?: (user: T) => MenuItem[];
  }>(),
  {
    isLoading: false,
    showHeader: true,
    title: "Users",
    description: "Manage accounts, roles, and user projects.",
  }
);

const floatingElements = ref<Record<UserId, any>>({});
const menuOpenStates = ref<Record<UserId, boolean>>({});

const closeMenu = (userId: UserId) => {
  if (floatingElements.value[userId]) {
    floatingElements.value[userId].isOpen = false;
  }
};

function handleAction(action?: () => void, userId?: UserId) {
  if (userId) {
    closeMenu(userId);
  }
  if (action) {
    action();
  }
}
</script>

<template>
  <div>
    <!-- Header -->
    <div v-if="showHeader" class="row padding middle-align no-margin bottom-margin">
      <div class="max">
        <h6 class="bold no-margin">{{ title }}</h6>
        <div v-if="description" class="small-text text-secondary">{{ description }}</div>
      </div>
      <slot name="header-actions" />
    </div>

    <!-- User List -->
    <div class="user-list-wrapper">
      <div v-if="isLoading && !users.length" class="center-align padding">
        <progress class="circle"></progress>
      </div>

      <div
        v-for="(user, index) in users"
        :key="user.id"
        class="row padding middle-align no-margin user-row"
        :class="{
          'border-top': index > 0 || showHeader,
          'menu-active': menuOpenStates[user.id]
        }"
      >
        <slot name="user-icon" :user="user">
          <i>account_circle</i>
        </slot>

        <div class="max">
          <div>{{ user.name }}</div>
          <div v-if="user.subtitle" class="small-text text-secondary">{{ user.subtitle }}</div>
        </div>

        <!-- Badges -->
        <slot name="badges" :user="user">
          <template v-if="getBadges">
            <span
              v-for="(badge, badgeIndex) in getBadges(user)"
              :key="badgeIndex"
              class="chip no-round no-border surface-container-highest no-margin role-chip"
              :class="[{ 'menu-active': menuOpenStates[user.id] }, badge.class]"
            >
              {{ badge.label }}
            </span>
          </template>
        </slot>

        <!-- More Menu -->
        <FloatingElement
          v-if="getMenuItems || $slots['menu-items']"
          :ref="(element) => { if (element) floatingElements[user.id] = element; }"
          align="left"
          class="relative action-menu"
          @toggle="menuOpenStates[user.id] = $event"
        >
          <template #button>
            <button tabindex="0" class="square transparent no-margin">
              <i>more_vert</i>
            </button>
          </template>
          <template #menu>
            <menu class="dropdown no-wrap no-round active surface-container">
              <slot name="menu-items" :user="user" :close="() => closeMenu(user.id)">
                <template v-if="getMenuItems">
                  <template v-for="(item, itemIndex) in getMenuItems(user)" :key="item.id || itemIndex">
                    <li v-if="item.isDivider"><hr /></li>
                    <button
                      v-else
                      @click.stop="handleAction(item.onClick, user.id)"
                      class="transparent no-round left-align"
                      :class="{ 'error-text': item.isDestructive }"
                    >
                      <i v-if="item.icon" class="small">{{ item.icon }}</i>
                      {{ item.label }}
                    </button>
                  </template>
                </template>
              </slot>
            </menu>
          </template>
        </FloatingElement>
      </div>
    </div>
  </div>
</template>

<style scoped>
.user-list-wrapper {
  border-radius: initial;
}
.user-row {
  transition: background-color 0.2s ease;
}
.user-row:hover {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 95%) !important;
}
.user-row.menu-active {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%) !important;
}
.role-chip {
  transition: background-color 0.2s ease;
}
.user-row:hover .role-chip {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%) !important;
}
.user-row.menu-active .role-chip {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%) !important;
}
.border-top {
  border-top: 1px solid var(--outline-variant);
}
.text-secondary {
  opacity: 0.7;
  font-size: 0.85rem;
}
.action-menu {
  margin-left: -0.25rem;
}
.dropdown hr {
  margin: 3px;
}
.dropdown i {
  margin-right: 0.5rem;
}
.dropdown {
  gap: 4px;
  box-shadow: none !important;
}
</style>
