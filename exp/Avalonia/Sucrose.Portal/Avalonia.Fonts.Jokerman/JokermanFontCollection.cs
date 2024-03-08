using Avalonia.Media.Fonts;
using System;

namespace Avalonia.Fonts.Jokerman
{
    public sealed class JokermanFontCollection : EmbeddedFontCollection
    {
        public JokermanFontCollection() : base(
            new Uri("fonts:Jokerman", UriKind.Absolute),
            new Uri("avares://Avalonia.Fonts.Jokerman/Assets", UriKind.Absolute))
        {
        }
    }
}
