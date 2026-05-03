**XhtmlExporter**

This module implements an HTML exporter that can be used with current Web browsers. Its HTML output already conforms to the stricter XHTML syntax, except for the XML header which is left out so that current browsers are not confused.

The (X)HTML elements/tags and attributes currently supported are: html, head, title, body, p, a(href, id), font(face, size, color), strong, em, u.

The exporter converts text. Embedded views are exported as asterisks. Link and target views (module *StdLinks*) are exceptions, they are turned into HTML links and anchors. Only links within documents are supported.

The exporter is preconfigured in *Config.Setup*:

*    Converters.Register("", "XhtmlExporter.ExportText", "TextViews.View", "html", {});*

