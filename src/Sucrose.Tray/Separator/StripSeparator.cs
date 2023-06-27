using Skylark.Enum;

namespace Sucrose.Tray.Separator
{
    public class StripSeparator
    {
        public ToolStripSeparator Strip;

        private readonly WindowsThemeType ThemeType;

        private readonly Color Dark = Color.FromArgb(55, 55, 55);
        private readonly Color Light = Color.FromArgb(240, 240, 240);

        public StripSeparator(WindowsThemeType ThemeType)
        {
            this.ThemeType = ThemeType;
            Strip = new ToolStripSeparator();
            Strip.Paint += StripSeparator_Paint;
        }

        private void StripSeparator_Paint(object sender, PaintEventArgs e)
        {
            ToolStripSeparator StripSeparator = sender as ToolStripSeparator;
            ContextMenuStrip MenuStrip = StripSeparator.Owner as ContextMenuStrip;

            e.Graphics.FillRectangle(new SolidBrush(Color.Transparent), new Rectangle(0, 0, StripSeparator.Width, StripSeparator.Height));

            using Pen Pen = new(ThemeType == WindowsThemeType.Dark ? Dark : Light, 1);

            e.Graphics.DrawLine(Pen, new Point(23, StripSeparator.Height / 2), new Point(MenuStrip.Width, StripSeparator.Height / 2));
        }
    }
}