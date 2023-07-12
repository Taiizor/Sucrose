namespace Sucrose.Tray.Table
{
    internal class LightColorTable : ProfessionalColorTable
    {
        Color ForegroundGray = Color.FromArgb(253, 253, 253);
        Color BackgroundGray = Color.FromArgb(248, 248, 248);

        public override Color ToolStripBorder => ForegroundGray;
        public override Color ToolStripDropDownBackground => ForegroundGray;
        public override Color ToolStripGradientBegin => ForegroundGray;
        public override Color ToolStripGradientEnd => ForegroundGray;
        public override Color ToolStripGradientMiddle => ForegroundGray;
        public override Color ImageMarginGradientBegin => BackgroundGray;
        public override Color ImageMarginGradientEnd => BackgroundGray;
        public override Color ImageMarginGradientMiddle => BackgroundGray;
        public override Color ImageMarginRevealedGradientBegin => ForegroundGray;
        public override Color ImageMarginRevealedGradientEnd => ForegroundGray;
        public override Color ImageMarginRevealedGradientMiddle => ForegroundGray;
        public override Color MenuItemSelected => ForegroundGray;
        public override Color MenuItemSelectedGradientBegin => ForegroundGray;
        public override Color MenuItemSelectedGradientEnd => ForegroundGray;
        public override Color MenuItemBorder => ForegroundGray;
        public override Color MenuBorder => BackgroundGray;
        public override Color ButtonCheckedGradientBegin => ForegroundGray;
    }
}