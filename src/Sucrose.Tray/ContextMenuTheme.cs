using System.Drawing.Drawing2D;

namespace Sucrose.Tray
{
    public class ContextMenuTheme
    {
        public class StripSeparatorCustom
        {
            public ToolStripSeparator stripSeparator;
            public StripSeparatorCustom()
            {
                stripSeparator = new ToolStripSeparator();
                stripSeparator.Paint += StripSeparator_Paint;
            }

            private void StripSeparator_Paint(object sender, PaintEventArgs e)
            {
                ToolStripSeparator stripSeparator = sender as ToolStripSeparator;
                ContextMenuStrip menuStrip = stripSeparator.Owner as ContextMenuStrip;
                e.Graphics.FillRectangle(new SolidBrush(Color.Transparent), new Rectangle(0, 0, stripSeparator.Width, stripSeparator.Height));
                using Pen pen = new(Color.FromArgb(55, 55, 55), 1);
                e.Graphics.DrawLine(pen, new Point(23, stripSeparator.Height / 2), new Point(menuStrip.Width, stripSeparator.Height / 2));
            }
        }

        public class RendererDark : ToolStripProfessionalRenderer
        {
            public RendererDark() : base(new DarkColorTable())
            {
            }

            protected override void OnRenderArrow(ToolStripArrowRenderEventArgs e)
            {
                e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
                Rectangle r = new(e.ArrowRectangle.Location, e.ArrowRectangle.Size);
                r.Inflate(-2, -6);
                e.Graphics.DrawLines(Pens.White, new Point[]{
                    new Point(r.Left, r.Top),
                    new Point(r.Right, r.Top + (r.Height /2)),
                    new Point(r.Left, r.Top+ r.Height)});
            }

            protected override void OnRenderItemCheck(ToolStripItemImageRenderEventArgs e)
            {
                e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
                Rectangle r = new(e.ImageRectangle.Location, e.ImageRectangle.Size);
                r.Inflate(-4, -6);
                e.Graphics.DrawLines(Pens.White, new Point[]{
                    new Point(r.Left, r.Bottom - (r.Height /2)),
                    new Point(r.Left + (r.Width /3),  r.Bottom),
                    new Point(r.Right, r.Top)});
            }

            protected override void OnRenderItemText(ToolStripItemTextRenderEventArgs e)
            {
                e.Item.ForeColor = Color.WhiteSmoke;
                base.OnRenderItemText(e);
            }

            protected override void OnRenderMenuItemBackground(ToolStripItemRenderEventArgs e)
            {
                if (!e.Item.Selected)
                {
                    base.OnRenderMenuItemBackground(e);
                }
                else
                {
                    SolidBrush fillColor = new(Color.FromArgb(75, 75, 75));
                    Pen borderColor = new(Color.FromArgb(75, 75, 75));
                    Rectangle rc = new(Point.Empty, e.Item.Size);
                    e.Graphics.FillRectangle(fillColor, rc);
                    e.Graphics.DrawRectangle(borderColor, 1, 0, rc.Width - 2, rc.Height - 1);
                    fillColor.Dispose();
                    borderColor.Dispose();
                }
            }
        }

        private class DarkColorTable : ProfessionalColorTable
        {
            Color foregroundGray = Color.FromArgb(43, 43, 43);
            Color backgroundGray = Color.FromArgb(50, 50, 50);
            public override Color ToolStripBorder => foregroundGray;
            public override Color ToolStripDropDownBackground => foregroundGray;
            public override Color ToolStripGradientBegin => foregroundGray;
            public override Color ToolStripGradientEnd => foregroundGray;
            public override Color ToolStripGradientMiddle => foregroundGray;
            public override Color ImageMarginGradientBegin => backgroundGray;
            public override Color ImageMarginGradientEnd => backgroundGray;
            public override Color ImageMarginGradientMiddle => backgroundGray;
            public override Color ImageMarginRevealedGradientBegin => foregroundGray;
            public override Color ImageMarginRevealedGradientEnd => foregroundGray;
            public override Color ImageMarginRevealedGradientMiddle => foregroundGray;
            public override Color MenuItemSelected => foregroundGray;
            public override Color MenuItemSelectedGradientBegin => foregroundGray;
            public override Color MenuItemSelectedGradientEnd => foregroundGray;
            public override Color MenuItemBorder => foregroundGray;
            public override Color MenuBorder => backgroundGray;
            public override Color ButtonCheckedGradientBegin => foregroundGray;
        }
    }
}