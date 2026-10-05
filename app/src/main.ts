import "@fontsource-variable/atkinson-hyperlegible-next";
import "./styles/tokens.css";
import { mount } from "svelte";
import App from "./App.svelte";
import { applyPalette, brandPalette, DEFAULT_BRAND } from "./brand.ts";

// First paint in the default preset; App switches to the saved one once settings load.
const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
applyPalette(document.documentElement, brandPalette(DEFAULT_BRAND, prefersDark ? "dark" : "light", ""));

mount(App, { target: document.getElementById("app")! });
