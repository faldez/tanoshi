(window.webpackJsonp=window.webpackJsonp||[]).push([[15],{411:function(t,a,s){"use strict";s.r(a);var n=s(56),e=Object(n.a)({},(function(){var t=this,a=t.$createElement,s=t._self._c||a;return s("ContentSlotsDistributor",{attrs:{"slot-key":t.$parent.slotKey}},[s("h1",{attrs:{id:"configuration-file"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#configuration-file"}},[t._v("#")]),t._v(" Configuration File")]),t._v(" "),s("p",[t._v("Tanoshi will look "),s("code",[t._v("config.yml")]),t._v(" in "),s("code",[t._v("$TANOSHI_HOME")]),t._v(" which defaults to "),s("code",[t._v("$HOME/.tanoshi")]),t._v(" on macos and linux, "),s("code",[t._v("C:\\Users\\<username>\\.tanoshi")]),t._v(" on windows. If config file doesn't exists, tanoshi will generate new file. Below is example configuration")]),t._v(" "),s("h2",{attrs:{id:"example"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#example"}},[t._v("#")]),t._v(" Example")]),t._v(" "),s("div",{staticClass:"language-yaml extra-class"},[s("pre",{pre:!0,attrs:{class:"language-yaml"}},[s("code",[s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Tanoshi base url without prefix '/', necessary to have link in notification")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("base_url")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v("  <your tanoshi url"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(">")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Port for tanoshi to server, default to 80, ignored in desktop version")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("port")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token number"}},[t._v("3030")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Absolute path to database")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("database_path")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" /absolute/path/to/database\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# JWT secret, any random value, changing this will render any active token invalid")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("secret")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" <16 alphanumeric characters"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(">")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Absolute path to where plugin is stored")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("plugin_path")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" /absolute/path/to/plugins\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Absolute path to local manga")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("local_path")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" /absolute/path/to/manga\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# you can multiple named directories for local manga")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# local_path:")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# - name: Local1")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("#   path: /path/to/local1")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# - name: Local2")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("#   path: /path/to/local2")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Absolute path to downloaded manga")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("download_path")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" /absolute/path/to/manga\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Periodic update interval in seconds, must be over 3600, set to 0 to disable")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("update_interval")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token number"}},[t._v("3600")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Automatically download chapter on update,set to true to enable")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("auto_download_chapters")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token boolean important"}},[t._v("false")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# GraphQL playground, set to true to enable")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("enable_playground")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token boolean important"}},[t._v("false")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Telegram token")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("telegram")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v("\n  "),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("name")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" <your bot name"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(">")]),t._v("\n  "),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("token")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" <your bot token"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(">")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Pushover")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("pushover")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v("\n  "),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("application_key")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" <your pushover application key"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(">")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token comment"}},[t._v("# Gotify")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("gotify")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v("\n  "),s("span",{pre:!0,attrs:{class:"token key atrule"}},[t._v("base_url")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" <URL to your server"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(">")]),t._v("\n")])])])])}),[],!1,null,null,null);a.default=e.exports}}]);