namespace Sucrose.Tray
{
    public class StripSeparatorDark
    {
        public ToolStripSeparator StripSeparator;

        public StripSeparatorDark()
        {
            StripSeparator = new ToolStripSeparator();
            StripSeparator.Paint += StripSeparator_Paint;
        }

        private void StripSeparator_Paint(object sender, PaintEventArgs e)
        {
            ToolStripSeparator StripSeparator = sender as ToolStripSeparator;
            ContextMenuStrip MenuStrip = StripSeparator.Owner as ContextMenuStrip;

            e.Graphics.FillRectangle(new SolidBrush(Color.Transparent), new Rectangle(0, 0, StripSeparator.Width, StripSeparator.Height));

            using Pen Pen = new(Color.FromArgb(55, 55, 55), 1);

            e.Graphics.DrawLine(Pen, new Point(23, StripSeparator.Height / 2), new Point(MenuStrip.Width, StripSeparator.Height / 2));
        }
    }
}