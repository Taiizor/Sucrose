namespace Sucrose.Tray
{
    public class StripSeparatorLight
    {
        public ToolStripSeparator StripSeparator;

        public StripSeparatorLight()
        {
            StripSeparator = new ToolStripSeparator();
            StripSeparator.Paint += StripSeparator_Paint;
        }

        private void StripSeparator_Paint(object sender, PaintEventArgs e)
        {
            ToolStripSeparator StripSeparator = sender as ToolStripSeparator;
            ContextMenuStrip MenuStrip = StripSeparator.Owner as ContextMenuStrip;

            e.Graphics.FillRectangle(new SolidBrush(Color.Transparent), new Rectangle(0, 0, StripSeparator.Width, StripSeparator.Height));

            using Pen Pen = new(Color.FromArgb(240, 240, 240), 1);

            e.Graphics.DrawLine(Pen, new Point(23, StripSeparator.Height / 2), new Point(MenuStrip.Width, StripSeparator.Height / 2));
        }
    }
}