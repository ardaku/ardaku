# Input
The (user) input service API

Input Packet:

bits  | Description
------|------------
0-15  | Device Type
16-23 | Input Type
24-31 | Sensor ID
32-63 | Value

Device types:
 0. None (Used to prevent bubbling of input to subprocesses)
 1. Keyboard (Keycode input)
 2. Pointer (Click/Vertical & Horizontal Scroll/Trackpad Swipes)
 3. Touchscreen (Gestures:Swipes/Taps)
 4. Gamepad/Joystick (W3 Standard Gamepad Mappings)
 5. Flightstick (Flightstick Mappings)
 6. Controller (Generic/Specialized Controller)
 7. Pedal (Accessiblity Pedal System)
 8. MIDI Controller (MIDI Input Controller)
 9. Text (Text input)

Input types:
 0. Button (Boolean: Pressed or Unpressed)
 1. Axis (Signed Axis - Returns to Zero)
 2. Pressure (Unsigned Axis - Pressure Button)
 3. Throttle (Signed Axis - Stationary Throttle)
 4. Wheel (Signed Axis - Spins)

## Keyboard
Keyboard events are based on keycode IDs (names after US Qwerty Layout).
Keycodes 0-63 are no-modifier keys, 64-127 are function keys, 128-191 are numpad
keys, and 192-255 are misc. extension keys.  After the / is how applications are
expected to interpret the key while the shift key is pressed (but not enforced,
hence why many of them have their own fn/ext keycodes).

Standard 64 Key:
 0. Backtick / Tilde
 1. One / Bang
 2. Two / At
 3. Three / Pound
 4. Four / Dollar
 5. Five / Percent
 6. Six / Caret
 7. Seven / Ampersand
 8. Eight / Asterisk
 9. Nine / ParensL
 10. Zero / ParensR
 11. Minus / Underscore
 12. Equal / Plus
 13. Backslash / Bar
 14. Tab / Untab
 15. Q
 16. W
 17. E
 18. R
 19. T
 20. Y
 21. U
 22. I
 23. O
 24. P
 25. BracketL / BraceL
 26. BracketR / BraceR
 27. Escape / Emoji
 28. A
 29. S
 30. D
 31. F
 32. G
 33. H
 34. J
 35. K
 36. L
 37. Semicolon / Colon
 38. Apostrophe / Quote
 39. Enter / Return (Previous line)
 40. ShiftL
 41. Z
 42. X
 43. C
 44. V
 45. B
 46. N
 47. M
 48. Less / Comma
 49. More / Period
 50. Slash / Question
 51. Up / (Select)
 52. ShiftR
 53. ControlL
 54. OptionL
 55. FunctionL (Left Function/Super/Meta/Command/Windows Key)
 56. Backspace / Delete
 57. Space / Non-Breaking Space
 58. Alt (Alternative Grapheme) / Compose
 59. OptionR
 60. ControlL
 61. Left / (Select)
 62. Down / (Select)
 63. Right / (Select)
 64. Lang (Next keyboard language layout) / (Previous)
 65. F1 / F13
 66. F2 / F14
 67. F3 / F15
 68. F4 / F16
 69. F5 / F17
 70. F6 / F18
 71. F7 / F19
 72. F8 / F20
 73. F9 / F21
 74. F10 / F22
 75. F11 / F23
 76. F12 / F24
 77. Script (Toggle language script) / Configure Language
 78. App (Next App) / (Previous)
 79. Ext (Extension Lock) / Scr (Scroll Lock)
 80. Display - Fn+W (Toggle Display on/off) / (Webcam on/off)
 81. Dimmer - Fn+E (Screen Brightness Down) / (Webcam Exposure Down)
 82. Brighter - Fn+R (Screen Brightness Up) / (Webcam Exposure Up)
 83. Airplane - Fn+T (Toggle Airplane Mode) / (Enable/Disable Bluetooth)
 84. WorkspaceTop - Fn+Y
 85. WorkspaceUp - Fn+U
 86. RESERVED - Fn+I
 87. RESERVED - Fn+O
 88. Screenshot - Fn+P (Print Screen - To File) / (Print Screen - To Clipboard)
 89. Insert - Fn+BracketL
 90. Delete - Fn+BrackerR
 91. Caps (Caps Lock) - Fn+Escape / (Numlock)
 92. App (Launch App) - Fn+A / Settings
 93. Mute (Volume Silent) - Fn+S / (Microphone Silent)
 94. Quieter (Volume Down) - Fn+D / (Microphone Quieter)
 95. Louder (Volume Up) - Fn+F / (Microphone Louder)
 96. Mirror (Toggle Display Mirroring) / (Display Configuration)
 97. Home - Fn+H / (Select)
 98. PageDown - Fn+J / (Select)
 99. PageUp - Fn+K / (Select)
 100. End - Fn+L / (Select)
 101. Workspace - Fn+Semicolon (Switch Workspace)
 102. Calculator - Fn+Apostrophe
 103. Solve (Numpad Enter) / (Ext Solve)
 104. RESERVED - /ShiftL
 105. Pause - Fn+Z
 106. Break - Fn+X
 107. Clear - Fn+C
 108. RESERVED - Fn+V
 109. Menu - Fn+B (Context Menu Key)
 110. WorkspaceDown - Fn+N
 111. WorkspaceBottom - FN+M
 112. ZoomOut - Fn+Less (Accessibility Key)
 113. ZoomIn - Fn+More (Accessibility Key)
 114. Help - Fn+Slash
 115. WindowUp (Split screen window up) - Fn+Up / Swap Window: Window Up
 116. RESERVED - /ShiftR
 117. RESERVED - /ControlL
 118. RESERVED - /OptionL
 119. FunctionR (Right Function/Super/Meta/Command/Windows Key)
 120. Prev - Fn+Backspace (Media Previous Track) / (Rewind)
 121. Play - Fn+Space (Media Play/Pause Track) / (Stop)
 122. Next - Fn+Alt (Media Next Track) / (Fast Forward)
 123. RESERVED - /OptionR
 124. RESERVED - /ControlL
 125. WindowLeft (Split screen window left) - Fn+Left / Swap Window: Window Left
 126. WindowDown (Split screen window down) - Fn+Down / Swap Window: Window Down
 127. WindowRight (Split screen window right) - Fn+Right / Swap Window: Window Right
 128. NumLock - Num+Backtick
 129. Num1 - Num+1
 130. Num2 - Num+2
 131. Num3 - Num+3
 132. Num4 - Num+4
 133. Num5 - Num+5
 134. Num6 - Num+6
 135. Num7 - Num+7
 136. Num8 - Num+8
 137. Num9 - Num+9
 138. Num0 - Num+0
 139. NumMinus - Num+Minus
 140. NumPlus - Num+Equal
 141. NumMultiply - Num+Backslash
 142. RESERVED /Num+Tab
 143. RESERVED /Num+Q
 144. RESERVED /Num+W
 145. RESERVED /Num+E
 146. RESERVED /Num+R
 147. RESERVED /Num+T
 148. RESERVED /Num+Y
 149. RESERVED /Num+U
 150. RESERVED /Num+I
 151. RESERVED /Num+O
 152. RESERVED /Num+P
 153. RESERVED /Num+BracketL
 154. RESERVED /Num+BracketR
 155. RESERVED /Num+Escape
 156. RESERVED /Num+A
 157. RESERVED /Num+S
 158. RESERVED /Num+D
 159. RESERVED /Num+F
 160. RESERVED /Num+G
 161. RESERVED /Num+H
 162. RESERVED /Num+J
 163. RESERVED /Num+K
 164. RESERVED /Num+L
 165. RESERVED /Num+Semicolon
 166. RESERVED /Num+Apostrophe
 167. RESERVED /Num+Enter
 168. RESERVED /Num+ShiftL
 169. RESERVED /Num+Z
 170. RESERVED /Num+X
 171. RESERVED /Num+C
 172. RESERVED /Num+V
 173. RESERVED /Num+B
 174. RESERVED /Num+N
 175. RESERVED /Num+M
 176. RESERVED /Num+Less
 177. NumDecimal - Num+Period
 178. NumDivide - Num+Slash
 179. RESERVED /Num+Up
 180. RESERVED /Num+ShiftR
 181. RESERVED /Num+ControlL
 182. RESERVED /Num+OptionL
 183. RESERVED /Num+FunctionR
 184. RESERVED /Num+Backspace
 185. RESERVED /Num+Space
 186. RESERVED /Num+Altgr
 187. RESERVED /Num+OptionR
 188. RESERVED /Num+ControlL
 189. RESERVED /Num+Left
 190. RESERVED /Num+Down
 191. RESERVED /Num+Right
 192. RESERVED /Ext+Backtick
 193. F13 - Ext+1
 194. F14 - Ext+2
 195. F15 - Ext+3
 196. F16 - Ext+4
 197. F17 - Ext+5
 198. F18 - Ext+6
 199. F19 - Ext+7
 200. F20 - Ext+8
 201. F21 - Ext+9
 202. F22 - Ext+0
 203. F23 - Ext+Minus
 204. F24 - Ext+Equal
 205. RESERVED /Ext+Backslash
 206. RESERVED /Ext+Tab
 207. RESERVED /Ext+Q
 208. RESERVED /Ext+W
 209. RESERVED /Ext+E
 210. RESERVED /Ext+R
 211. RESERVED /Ext+T
 212. RESERVED /Ext+Y
 213. RESERVED /Ext+U
 214. RESERVED /Ext+I
 215. RESERVED /Ext+O
 216. RESERVED /Ext+P
 217. RESERVED /Ext+BracketL
 218. RESERVED /Ext+BracketR
 219. Power /Ext+Escape
 220. RESERVED /Ext+A
 221. RESERVED /Ext+S
 222. RESERVED /Ext+D
 223. RESERVED /Ext+F
 224. RESERVED /Ext+G
 225. RESERVED /Ext+H
 226. RESERVED /Ext+J
 227. RESERVED /Ext+K
 228. RESERVED /Ext+L
 229. RESERVED /Ext+Semicolon
 230. RESERVED /Ext+Apostrophe
 231. RESERVED /Ext+Enter
 232. RESERVED /Ext+ShiftL
 233. RESERVED /Ext+Z
 234. RESERVED /Ext+X
 235. RESERVED /Ext+C
 236. RESERVED /Ext+V
 237. RESERVED /Ext+B
 238. RESERVED /Ext+N
 239. RESERVED /Ext+M
 240. RESERVED /Ext+Less
 241. RESERVED /Ext+More
 242. RESERVED /Ext+Slash
 243. RESERVED /Ext+Up
 244. RESERVED /Ext+ShiftR
 245. RESERVED /Ext+ControlL
 246. RESERVED /Ext+OptionL
 247. RESERVED /Ext+FunctionR
 248. RESERVED /Ext+Backspace
 249. RESERVED /Ext+Space
 250. RESERVED /Ext+Altgr
 251. RESERVED /Ext+OptionR
 252. RESERVED /Ext+ControlL
 253. RESERVED /Ext+Left
 254. RESERVED /Ext+Down
 255. RESERVED /Ext+Right
