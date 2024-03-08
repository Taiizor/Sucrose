using Avalonia.Media.Fonts;
using System;

namespace Avalonia.Fonts.SegoeUI
{
    public sealed class SegoeUIFontCollection1 : EmbeddedFontCollection
    {
        public SegoeUIFontCollection1() : base(
            new Uri("fonts:Segoe", UriKind.Absolute),
            new Uri("avares://Avalonia.Fonts.SegoeUI/Assets", UriKind.Absolute))
        {
        }
    }

    public sealed class SegoeUIFontCollection2 : EmbeddedFontCollection
    {
        public SegoeUIFontCollection2() : base(
            new Uri("fonts:SegoeUI", UriKind.Absolute),
            new Uri("avares://Avalonia.Fonts.SegoeUI/Assets", UriKind.Absolute))
        {
        }
    }

    public sealed class SegoeUIFontCollection3 : EmbeddedFontCollection
    {
        public SegoeUIFontCollection3() : base(
            new Uri("fonts:Segoe UI", UriKind.Absolute),
            new Uri("avares://Avalonia.Fonts.SegoeUI/Assets", UriKind.Absolute))
        {
        }
    }
}
