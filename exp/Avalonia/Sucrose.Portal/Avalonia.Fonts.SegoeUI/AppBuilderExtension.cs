using Avalonia.Fonts.SegoeUI;

namespace Avalonia
{
    public static class AppBuilderExtension
    {
        public static AppBuilder WithSegoeUIFont(this AppBuilder appBuilder)
        {
            return appBuilder.ConfigureFonts(fontManager =>
            {
                fontManager.AddFontCollection(new SegoeUIFontCollection1());
                fontManager.AddFontCollection(new SegoeUIFontCollection2());
                fontManager.AddFontCollection(new SegoeUIFontCollection3());
            });
        }
    }
}
