using Avalonia.Fonts.Jokerman;

namespace Avalonia
{
    public static class AppBuilderExtension
    {
        public static AppBuilder WithJokermanFont(this AppBuilder appBuilder)
        {
            return appBuilder.ConfigureFonts(fontManager =>
            {
                fontManager.AddFontCollection(new JokermanFontCollection());
            });
        }
    }
}
