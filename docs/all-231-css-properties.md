# Complete Catalog: 231 CSS Properties in `tej-ui`

## Executive Summary

`tej-ui` now implements a strongly-typed, zero-allocation CSS property system encompassing **all 231 foundational, responsive, typography, SVG, animation, layout, and modern CSS properties** (Part 1's 106 properties + Part 2's 125 properties).

Located in [`crates/tej-view/src/css/`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/), these properties are organized into 10 decoupled submodules, with [`TOTAL_CSS_PROPERTIES = 231`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/mod.rs#L24) and [`CssStyleSheet`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/mod.rs#L27).

---

## Complete Taxonomy & Property Breakdown (231 Properties)

### 1. Box Model & Sizing (14 Properties) — [`box_sizing.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/box_sizing.rs)
1. `box-sizing`
2. `width`
3. `max-width`
4. `height`
5. `min-height`
6. `min-width`
7. `max-height`
8. `aspect-ratio`
9. `inline-size`
10. `block-size`
11. `min-inline-size`
12. `max-inline-size`
13. `min-block-size`
14. `max-block-size`

### 2. Spacing, Offsets & Insets (31 Properties) — [`spacing.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/spacing.rs)
15. `margin`
16. `margin-top`
17. `margin-bottom`
18. `margin-left`
19. `margin-right`
20. `margin-block`
21. `margin-block-start`
22. `margin-block-end`
23. `margin-inline`
24. `margin-inline-start`
25. `margin-inline-end`
26. `padding`
27. `padding-top`
28. `padding-bottom`
29. `padding-left`
30. `padding-right`
31. `padding-block`
32. `padding-block-start`
33. `padding-block-end`
34. `padding-inline`
35. `padding-inline-start`
36. `padding-inline-end`
37. `position`
38. `top`
39. `right`
40. `bottom`
41. `left`
42. `inset`
43. `inset-block`
44. `inset-inline`
45. `z-index`

### 3. Flexbox & CSS Grid (29 Properties) — [`flex_grid.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/flex_grid.rs)
46. `display`
47. `flex-direction`
48. `justify-content`
49. `align-items`
50. `gap`
51. `row-gap`
52. `column-gap`
53. `flex-wrap`
54. `flex`
55. `flex-grow`
56. `flex-shrink`
57. `flex-basis`
58. `align-self`
59. `order`
60. `align-content`
61. `place-content`
62. `place-items`
63. `place-self`
64. `justify-items`
65. `justify-self`
66. `grid-template-columns`
67. `grid-template-rows`
68. `grid-column`
69. `grid-row`
70. `grid-template-areas`
71. `grid-area`
72. `grid-auto-flow`
73. `grid-auto-columns`
74. `grid-auto-rows`

### 4. Typography & Internationalization (34 Properties) — [`typography.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/typography.rs)
75. `color`
76. `font-family`
77. `font-size`
78. `font-weight`
79. `line-height`
80. `text-align`
81. `text-decoration`
82. `text-decoration-color`
83. `text-decoration-thickness`
84. `text-underline-offset`
85. `letter-spacing`
86. `text-transform`
87. `white-space`
88. `list-style`
89. `font-style`
90. `text-overflow`
91. `word-break`
92. `overflow-wrap`
93. `vertical-align`
94. `writing-mode`
95. `direction`
96. `text-orientation`
97. `hyphens`
98. `text-wrap`
99. `word-spacing`
100. `text-indent`
101. `text-shadow`
102. `font-variant-numeric`
103. `font-feature-settings`
104. `font-optical-sizing`
105. `tab-size`
106. `caret-color`
107. `counter-reset`
108. `counter-increment`

### 5. Borders, Radii, Shadows & Effects (25 Properties) — [`borders_effects.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/borders_effects.rs)
109. `border`
110. `border-top`
111. `border-bottom`
112. `border-left`
113. `border-right`
114. `border-block`
115. `border-inline`
116. `border-color`
117. `border-width`
118. `border-style`
119. `border-radius`
120. `border-start-start-radius`
121. `border-end-end-radius`
122. `box-shadow`
123. `opacity`
124. `visibility`
125. `outline`
126. `outline-offset`
127. `mix-blend-mode`
128. `filter`
129. `backdrop-filter`
130. `isolation`
131. `will-change`
132. `contain`
133. `content-visibility`

### 6. Backgrounds & Masking (16 Properties) — [`backgrounds.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/backgrounds.rs)
134. `background-color`
135. `background`
136. `background-image`
137. `background-size`
138. `background-position`
139. `background-repeat`
140. `background-attachment`
141. `background-clip`
142. `background-origin`
143. `background-blend-mode`
144. `mask-image`
145. `mask-size`
146. `mask-position`
147. `mask-repeat`
148. `mask-composite`
149. `clip-path`

### 7. Motion, Animations & 3D GPU Transforms (22 Properties) — [`motion.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/motion.rs)
150. `transition`
151. `transition-property`
152. `transition-duration`
153. `transition-timing-function`
154. `transition-delay`
155. `transform`
156. `transform-origin`
157. `animation`
158. `animation-name`
159. `animation-duration`
160. `animation-timing-function`
161. `animation-delay`
162. `animation-iteration-count`
163. `animation-direction`
164. `animation-fill-mode`
165. `animation-play-state`
166. `animation-timeline`
167. `scroll-timeline`
168. `perspective`
169. `perspective-origin`
170. `transform-style`
171. `backface-visibility`

### 8. Scrolling, Scroll-Snap & Interaction (18 Properties) — [`interaction.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/interaction.rs)
172. `overflow`
173. `overflow-x`
174. `overflow-y`
175. `scroll-behavior`
176. `scroll-margin-top`
177. `scroll-padding-top`
178. `scroll-snap-type`
179. `scroll-snap-align`
180. `scroll-snap-stop`
181. `overscroll-behavior`
182. `scrollbar-width`
183. `scrollbar-color`
184. `scrollbar-gutter`
185. `cursor`
186. `pointer-events`
187. `user-select`
188. `touch-action`
189. `resize`

### 9. Modern Modules, SVG, Tables & Paged Media (42 Properties) — [`modern.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/src/css/modern.rs)
190. `anchor-name`
191. `position-anchor`
192. `position-area`
193. `view-transition-name`
194. `color-scheme`
195. `forced-color-adjust`
196. `accent-color`
197. `appearance`
198. `content`
199. `object-fit`
200. `object-position`
201. `container-type`
202. `container-name`
203. `container`
204. `table-layout`
205. `border-collapse`
206. `border-spacing`
207. `caption-side`
208. `empty-cells`
209. `column-count`
210. `column-width`
211. `columns`
212. `column-rule`
213. `column-span`
214. `break-inside`
215. `break-before`
216. `orphans`
217. `widows`
218. `fill`
219. `fill-opacity`
220. `fill-rule`
221. `stroke`
222. `stroke-width`
223. `stroke-opacity`
224. `stroke-dasharray`
225. `stroke-dashoffset`
226. `stroke-linecap`
227. `stroke-linejoin`
228. `vector-effect`
229. `paint-order`
230. `shape-outside`
231. `shape-margin`

---

## Verification Summary

* Total properties: **231** (`TOTAL_CSS_PROPERTIES = 231`)
* Tests: [`crates/tej-view/tests/css_properties_tests.rs`](file:///home/yeab/Projects/Work/tej-ui/crates/tej-view/tests/css_properties_tests.rs) (100% pass)
* Quality Gates: All pass with 0 warnings (`cargo fmt`, `cargo clippy`, `cargo test`).
