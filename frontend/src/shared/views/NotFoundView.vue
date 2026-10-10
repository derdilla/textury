<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";

const props = withDefaults(
  defineProps<{
    type?: "generic" | "project" | "settings";
  }>(),
  {
    type: "generic",
  }
);

const router = useRouter();

const config = computed(() => {
  switch (props.type) {
    case "project":
      return {
        icon: "folder_off",
        title: "Project not found",
        description: "The project you are looking for does not exist, or you don't have permission to access it.",
        buttonText: "Go to Projects",
        buttonIcon: "arrow_back",
        action: () => router.push({ name: "projects" }),
      };
    case "settings":
      return {
        icon: "settings",
        title: "Settings page not found",
        description: "The settings tab or page you are trying to access does not exist.",
        buttonText: "Go to Settings",
        buttonIcon: "arrow_back",
        action: () => router.push({ name: "settings" }),
      };
    case "generic":
    default:
      return {
        icon: "error",
        title: "Page not found",
        description: "The page you are looking for doesn't exist or has been moved.",
        buttonText: "Back to Home",
        buttonIcon: "home",
        action: () => router.push({ name: "login" }),
      };
  }
});
</script>

<template>
  <main class="responsive no-padding no-margin max center-align middle-align">
    <article class="medium center-align box margin no-elevate">
      <div class="content-box">
        <i class="extra">{{ config.icon }}</i>
        <h5 class="bold responsive-title">{{ config.title }}</h5>
        <h6 class="small responsive-description">{{ config.description }}</h6>

        <div class="empty-actions">
          <button tabindex="0" class="chip medium primary no-border" @click="config.action">
            <i>{{ config.buttonIcon }}</i> {{ config.buttonText }}
          </button>
        </div>
      </div>
    </article>
  </main>
</template>

<style scoped>
.box {
  padding: 2.5rem !important;
}

.content-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.responsive-title {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.responsive-description {
  margin-top: 0;
  margin-bottom: 0;
}

.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1.5rem;
}

@media (max-width: 600px) {
  .responsive-title {
    font-size: 1.25rem;
  }

  .responsive-description {
    font-size: 0.8rem;
  }
}
</style>
