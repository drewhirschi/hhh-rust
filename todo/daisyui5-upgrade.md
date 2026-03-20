# DaisyUI 5 Upgrade TODO

Review of all templates against DaisyUI 5 + Tailwind CSS 4 patterns.

## 1. Upgrade CDN to DaisyUI 5 + Tailwind CSS 4 (Critical)

**File:** `templates/base.html:7-8`

Currently loads DaisyUI **4.12.14** with Tailwind CSS **v3** CDN:
```html
<link href="https://cdn.jsdelivr.net/npm/daisyui@4.12.14/dist/full.min.css" rel="stylesheet">
<script src="https://cdn.tailwindcss.com"></script>
```
Should be:
```html
<link href="https://cdn.jsdelivr.net/npm/daisyui@5" rel="stylesheet" type="text/css" />
<script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4"></script>
```

## 2. Stats missing wrapper `stats` component

**Files:** `templates/dashboard/member.html`, `templates/dashboard/employee.html`, `templates/dashboard/admin.html`

Every dashboard uses bare `<div class="stat ...">` elements directly in a CSS grid. DaisyUI 5 expects a `stats` wrapper:
```html
<div class="stats">
  <div class="stat">...</div>
</div>
```
Currently the stats are individually placed in grid cells, bypassing the DaisyUI stats component entirely.

## 3. Forms use deprecated v4 `label`/`form-control` pattern

**Files:** All form templates — `auth/login.html`, `auth/register.html`, `account/profile.html`, `account/change_password.html`, `employee/class_form.html`, `employee/schedule_form.html`, `admin/user_form.html`, `admin/settings.html`

Old v4 pattern used everywhere:
```html
<div class="form-control">
  <label class="label"><span class="label-text">Email</span></label>
  <input class="input input-bordered w-full" />
</div>
```
DaisyUI 5 pattern:
```html
<label class="input">
  <span class="label">Email</span>
  <input type="email" placeholder="..." />
</label>
```
The `form-control`, `label-text`, `label-text-alt`, and `input-bordered` classes are all v4. In v5, `input` is the wrapper and `label` is a simple span class.

## 4. `textarea-bordered` and `select-bordered` are v4 classes

**Files:** `employee/class_form.html`, `employee/schedule_form.html`, `admin/user_form.html`

`textarea textarea-bordered` and `select select-bordered` don't exist in v5. The `textarea` and `select` components in v5 have `ghost` style variants but no `-bordered` modifier.

## 5. Navbar not using `navbar-start`/`navbar-center`/`navbar-end`

**File:** `templates/partials/nav.html`

Uses raw flexbox (`flex-none`, `flex-1`, `flex-none`) instead of proper DaisyUI navbar parts:
```html
<div class="navbar">
  <div class="navbar-start">...</div>
  <div class="navbar-center">...</div>
  <div class="navbar-end">...</div>
</div>
```

## 6. Footer not using DaisyUI 5 pattern correctly

**File:** `templates/base.html:26`

Uses `footer footer-center p-4 bg-base-300`. The v5 docs suggest `bg-base-200` and the `footer-horizontal`/`footer-vertical` direction classes for responsiveness.

## 7. Dropdown uses CSS focus pattern (v4 tabindex hack)

**File:** `templates/partials/nav.html:14-29`

Uses the old tabindex-based dropdown:
```html
<div class="dropdown dropdown-end">
  <div tabindex="0" role="button">...</div>
  <ul tabindex="0" class="dropdown-content menu ...">
```
DaisyUI 5 prefers `<details>`/`<summary>` or the popover API:
```html
<details class="dropdown dropdown-end">
  <summary class="btn btn-ghost">...</summary>
  <ul class="dropdown-content menu ...">
```

## 8. Cards use manual `shadow-xl` instead of DaisyUI card modifiers

**Files:** Most templates using cards

Cards use `card bg-base-100 shadow-xl` with manual styling. DaisyUI 5 cards have proper style modifiers: `card-border`, `card-dash`, and size classes (`card-sm`, `card-md`, etc.).

## 9. `badge-ghost` does not exist in DaisyUI 5

**Files:** `classes/schedule.html:64`, `bookings/_booking_row.html:18`, `employee/invites.html:51`, `employee/classes.html:60`, `admin/users.html:53`, `employee/rosters.html:72`

DaisyUI 5 badge styles are: `badge-outline`, `badge-dash`, `badge-soft`. Colors are: `badge-neutral`, `badge-primary`, `badge-secondary`, `badge-accent`, `badge-info`, `badge-success`, `badge-warning`, `badge-error`. Replace `badge-ghost` with `badge-neutral` or `badge-outline`.

## 10. `btn-disabled` class on non-disabled button

**File:** `templates/classes/detail.html:69`

Uses `class="btn btn-disabled"`. In DaisyUI 5, prefer the native `disabled` attribute on the `<button>` element instead.

## 11. Alert missing `role="alert"` attribute

**Files:** `templates/base.html:21`, `auth/login.html:13`, `auth/register.html:13`

DaisyUI 5 alert syntax expects `role="alert"`:
```html
<div role="alert" class="alert {MODIFIER}">{CONTENT}</div>
```
The flash alert in `base.html` and error alerts in auth forms should include this attribute.
