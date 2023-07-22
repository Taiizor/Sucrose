using SEWTT = Skylark.Enum.WindowsThemeType;

namespace Sucrose.Shared.Launcher.Separator
{
    public class StripSeparator
    {
        public ToolStripSeparator Strip;

        private readonly SEWTT ThemeType;

        private readonly Color Dark = Color.FromArgb(55, 55, 55);
        private readonly Color Light = Color.FromArgb(240, 240, 240);

        public StripSeparator(SEWTT ThemeType)
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

            using Pen Pen = new(ThemeType == SEWTT.Dark ? Dark : Light, 1);

            e.Graphics.DrawLine(Pen, new Point(23, StripSeparator.Height / 2), new Point(MenuStrip.Width, StripSeparator.Height / 2));
        }
    }
}