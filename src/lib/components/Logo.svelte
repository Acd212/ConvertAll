<script lang="ts">
  let { size = 36 }: { size?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 1024 1024"
  fill="none"
  aria-hidden="true"
>
  <rect width="1024" height="1024" rx="232" fill="#171412" />
  <rect
    x="26"
    y="26"
    width="972"
    height="972"
    rx="210"
    stroke="#2A2522"
    stroke-width="12"
  />
  <path
    d="M302 396a260 260 0 0 1 430-64"
    stroke="#F97316"
    stroke-width="68"
    stroke-linecap="round"
  />
  <path
    d="M742 300l8 128-128-8"
    stroke="#F97316"
    stroke-width="68"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
  <path
    d="M722 628a260 260 0 0 1-430 64"
    stroke="#F5F5F4"
    stroke-width="68"
    stroke-linecap="round"
  />
  <path
    d="M282 724l-8-128 128 8"
    stroke="#F5F5F4"
    stroke-width="68"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>
