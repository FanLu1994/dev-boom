<script setup lang="ts">
defineProps<{
  searchText: string;
  favoritesOnly: boolean;
}>();

defineEmits<{
  "update:searchText": [value: string];
  "update:favoritesOnly": [value: boolean];
}>();
</script>

<template>
  <section class="toolbar glass">
    <input
      :value="searchText"
      class="input"
      placeholder="搜索项目名 / 标签 / 路径..."
      @input="$emit('update:searchText', ($event.target as HTMLInputElement).value)"
    />
    <label class="chip check">
      <input
        :checked="favoritesOnly"
        type="checkbox"
        @change="$emit('update:favoritesOnly', ($event.target as HTMLInputElement).checked)"
      />
      仅收藏
    </label>
  </section>
</template>
